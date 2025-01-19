use tonic::{transport::Server, Request, Response, Status};
use security::{SecurityEvent, SecurityCommand, Notification, security_service_server::{SecurityService, SecurityServiceServer}};
use influxdb2::{Client as InfluxDBClient, FromDataPoint};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod security {
    tonic::include_proto!("security");
}

#[derive(Debug, FromDataPoint)]
struct SecurityEventPoint {
    #[measurement = "security_events"]
    measurement: String,
    #[tag]
    device_id: String,
    #[field]
    event_type: String,
    #[timestamp]
    time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct MySecurityService {
    influxdb_client: InfluxDBClient,
    armed: Arc<Mutex<bool>>,
}

#[tonic::async_trait]
impl SecurityService for MySecurityService {
    async fn send_event(&self, request: Request<SecurityEvent>) -> Result<Response<Notification>, Status> {
        let event = request.into_inner();
        let mut armed = self.armed.lock().await;

        if *armed {
            let data_point = SecurityEventPoint {
                measurement: "security_events".to_string(),
                device_id: event.device_id,
                user_id: event.user_id,
                event_type: event.event.event_type,
                time: Utc::now(),
            };

            //write to influxdb
            self.influxdb_client.write("security-bucket", "my-org", vec![data_point]).await.map_err(|e| Status::internal(format!("Failed to write to InfluxDB: {}", e)))?;

            Ok(Response::new(Notification {
                user_id: event.user_id,
                message: format!("Security event recorded: {}", event.event_type),
            }))
        } else {
            Ok(Response::new(Notification {
                user_id: event.user_id,
                message: "System is disarmed, event ignored.".to_string(),
            }))
        }
    }

    async fn control_security(&self, request: Request<SecurityCommand>) -> Result<Response<Notification>, Status> {
        let command = request.into_inner();
        let mut armed = self.armed.lock().await;

        match command.command.as_str() {
            "arm" => {
                *armed = true;
                Ok(Response::new(Notification{
                    
                    user_id: "".to_string(), //assuming no specific user
                    message: "Security system armed."to_string(),
                }))
            }
            "disarm" => {
                *armed = false;
                Ok(Response::new(Notification{
                    user_id: "".to_string(),
                    message: "Security system disarmed.".to_string(),
                }))
            }
            _=>
            Err(Status::invalid_argument("Invalid command")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let influxdb_client = InfluxDBClient::new("http://localhost:8086", "my-org", "my-token",);
    let service = MySecurityService {
        influxdb_client, armed: Arc::new(Mutex::new(false)),
    };

    Server::builder().add_service(SecurityServiceServer::new(service)).serve(addr).await?;

    Ok(())
}
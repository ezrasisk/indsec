use tonic::{transport::Channel, Request};
use srcurity::{SecurityEvent, SecurityCommand};
use std::time::Duration;

pub mod security {
    tonic::include_proto!("security");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error:Error>> {
    let channel = Channel::from_static("http://[::1]:50051").connect_timeout(Duration::from_secs(5)).connect().await?;
    let mut client = security::security_service_client::SecurityServiceClient::new(channel);

    //arm system
    let arm_response = client.control_security(Request::new(SecurityCommand {
        command: "arm".to_string(),
    })).await?;
    println!("Arm response: {:?}", arm_response.into_inner());

    //security event simulation
    let event_response = client.send_event(Request::new(SecurityEvent {
        device_id: "sensor_001".to_string(),
        user_id: "user_123".to_string(),
        event_type: "motion".to_string(),
    })).await?;
    println!("Event response: {:?}", event_response.into_inner());

    //system disarm
    let disarm_response = client.control_security(Request::new(SecurityCommand {
        command: "disarm".to_string(),
    })).await?;
    println!("Disarm response: {:?}", disarm_response.into_inner());

    Ok(())
}
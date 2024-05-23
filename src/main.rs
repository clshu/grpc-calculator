use proto::calculator_server::{Calculator, CalculatorServer};
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("calculator");
}

#[derive(Debug, Default)]
struct CalculatorService {}

#[tonic::async_trait]
impl Calculator for CalculatorService {
    async fn add(
        &self,
        request: tonic::Request<proto::CalculationRequest>,
    ) -> Result<tonic::Response<proto::CalculationResponse>, tonic::Status> {
        println!("Got a request: {:?}", request);

        let input = request.get_ref();

        let response = proto::CalculationResponse {
            result: input.a + input.b,
        };

        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    let calculator = CalculatorService::default();

    println!("CalculatorServer listening on {}", addr);

    Server::builder()
        .add_service(CalculatorServer::new(calculator))
        .serve(addr)
        .await?;

    Ok(())
}

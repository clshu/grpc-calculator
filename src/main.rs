// use proto::admin_server::Admin;
use proto::{
    admin_server::{Admin, AdminServer},
    calculator_server::{Calculator, CalculatorServer},
};
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("calculator");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("calculator_descriptor");
}

type State = std::sync::Arc<tokio::sync::RwLock<u64>>;

#[derive(Debug, Default)]
struct CalculatorService {
    state: State,
}

impl CalculatorService {
    fn new(state: State) -> Self {
        Self { state }
    }
    async fn increment_counter(&self) {
        let mut count = self.state.write().await;
        *count += 1;
        println!("Counter: {}", *count);
    }
}

#[tonic::async_trait]
impl Calculator for CalculatorService {
    async fn add(
        &self,
        request: tonic::Request<proto::CalculationRequest>,
    ) -> Result<tonic::Response<proto::CalculationResponse>, tonic::Status> {
        println!("Got a request: {:?}", request);
        self.increment_counter().await;

        let input = request.get_ref();

        let response = proto::CalculationResponse {
            result: input.a + input.b,
        };

        Ok(tonic::Response::new(response))
    }

    async fn divide(
        &self,
        request: tonic::Request<proto::CalculationRequest>,
    ) -> Result<tonic::Response<proto::CalculationResponse>, tonic::Status> {
        println!("Got a request: {:?}", request);
        self.increment_counter().await;

        let input = request.get_ref();

        if input.b == 0 {
            return Err(tonic::Status::invalid_argument("Cannot divide by zero"));
        }

        let response = proto::CalculationResponse {
            result: input.a / input.b,
        };

        Ok(tonic::Response::new(response))
    }
}

struct AdminService {
    state: State,
}

impl AdminService {
    fn new(state: State) -> Self {
        Self { state }
    }
}
#[tonic::async_trait]
impl Admin for AdminService {
    async fn get_request_count(
        &self,
        _: tonic::Request<proto::GetCountRequest>,
    ) -> Result<tonic::Response<proto::GetCountResponse>, tonic::Status> {
        let count = self.state.read().await;
        let response = proto::GetCountResponse { count: *count };
        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    let state = State::default();

    let admin = AdminService::new(state.clone());

    let calculator = CalculatorService::new(state.clone());

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    println!("CalculatorServer listening on {}", addr);

    Server::builder()
        .add_service(service)
        .add_service(CalculatorServer::new(calculator))
        .add_service(AdminServer::new(admin))
        .serve(addr)
        .await?;

    Ok(())
}

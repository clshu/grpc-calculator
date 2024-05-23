// use admin_server::Admin;
use proto::{
    admin_server::{Admin, AdminServer},
    calculator_server::{Calculator, CalculatorServer},
    CalculationRequest, CalculationResponse, GetCountRequest, GetCountResponse,
    FILE_DESCRIPTOR_SET,
};
use tonic::async_trait;
use tonic::{metadata::MetadataValue, transport::Server, Request, Response, Status};

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

#[async_trait]
impl Calculator for CalculatorService {
    async fn add(
        &self,
        request: Request<CalculationRequest>,
    ) -> Result<Response<CalculationResponse>, Status> {
        println!("Got a request: {:?}", request);
        self.increment_counter().await;

        let input = request.get_ref();

        let response = CalculationResponse {
            result: input.a + input.b,
        };

        Ok(Response::new(response))
    }

    async fn divide(
        &self,
        request: Request<CalculationRequest>,
    ) -> Result<Response<CalculationResponse>, Status> {
        println!("Got a request: {:?}", request);
        self.increment_counter().await;

        let input = request.get_ref();

        if input.b == 0 {
            return Err(Status::invalid_argument("Cannot divide by zero"));
        }

        let response = CalculationResponse {
            result: input.a / input.b,
        };

        Ok(Response::new(response))
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
#[async_trait]
impl Admin for AdminService {
    async fn get_request_count(
        &self,
        _: Request<GetCountRequest>,
    ) -> Result<Response<GetCountResponse>, Status> {
        let count = self.state.read().await;
        let response = GetCountResponse { count: *count };
        Ok(Response::new(response))
    }
}

fn check_auth(request: Request<()>) -> Result<Request<()>, Status> {
    let secret: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();

    let token = request.metadata().get("authorization");

    if let Some(token) = token {
        if token == secret {
            Ok(request)
        } else {
            Err(Status::unauthenticated("Invalid token"))
        }
    } else {
        Err(Status::unauthenticated("Missing token"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    let state = State::default();

    let admin = AdminService::new(state.clone());

    let calculator = CalculatorService::new(state.clone());

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;

    println!("CalculatorServer listening on {}", addr);

    Server::builder()
        .add_service(service)
        .add_service(CalculatorServer::new(calculator))
        .add_service(AdminServer::with_interceptor(admin, check_auth))
        .serve(addr)
        .await?;

    Ok(())
}

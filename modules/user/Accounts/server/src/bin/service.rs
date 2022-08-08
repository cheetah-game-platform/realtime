use std::fmt::format;

use include_dir::include_dir;

use cheetah_user_accounts::grpc::run_grpc_server;
use cheetah_user_accounts::postgres::create_postgres_pool;
use ydb_steroids::builder::YdbClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("accounts");

	let db_port = cheetah_libraries_microservice::get_env("POSTGRES_PORT");
	let pg_pool = create_postgres_pool(
		cheetah_libraries_microservice::get_env("POSTGRES_DB").as_str(),
		cheetah_libraries_microservice::get_env("POSTGRES_USER").as_str(),
		cheetah_libraries_microservice::get_env("POSTGRES_PASSWORD").as_str(),
		cheetah_libraries_microservice::get_env("POSTGRES_HOST").as_str(),
		db_port
			.parse()
			.expect(format!("POSTGRES_PORT is wrong {:?}", db_port).as_str()),
	)
	.await;

	// ключи для генерации токенов
	let jwt_public_key = cheetah_libraries_microservice::get_env("JWT_PUBLIC_KEY");
	let jwt_private_key = cheetah_libraries_microservice::get_env("JWT_PRIVATE_KEY");
	let google_client_id = std::env::var("AUTH_GOOGLE_CLIENT_ID").ok();

	run_grpc_server(jwt_public_key, jwt_private_key, pg_pool, google_client_id).await;
	Ok(())
}

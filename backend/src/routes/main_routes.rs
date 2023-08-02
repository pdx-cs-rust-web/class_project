use axum::response::Response;
use axum::routing::*;
use axum::Router;
use http::StatusCode;
use hyper::Body;
use sqlx::PgPool;
use tracing::info;

use crate::db::Store;
use crate::handlers::main_handlers;
use crate::handlers::main_handlers::root;
use crate::layers;
use crate::routes::comment_routes::comment_routes;

pub async fn app(pool: PgPool) -> Router {
    let db = Store::with_pool(pool);

    info!("Seeded database");

    let (cors_layer, trace_layer) = layers::get_layers();

    Router::new()
        // The router matches these FROM TOP TO BOTTOM explicitly!
        .route("/", get(root))
        .route("/questions", get(main_handlers::get_questions))
        .route(
            "/question/:question_id",
            get(main_handlers::get_question_by_id),
        )
        .route("/question", post(main_handlers::create_question))
        .route("/question", put(main_handlers::update_question))
        .route("/question", delete(main_handlers::delete_question))
        .route("/answer", post(main_handlers::create_answer))
        .route("/users", post(main_handlers::register))
        .route("/login", post(main_handlers::login))
        .route("/*_", get(handle_404))
        //.route("/comments", post(create_new_comment))
        .merge(comment_routes())
        //.nest_service("/comments", comment_routes(db.clone()))
        .layer(cors_layer)
        .layer(trace_layer)
        .with_state(db.clone())
}

async fn handle_404() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("The requested page could not be found"))
        .unwrap()
}

/*
 Generic Router creation for merging.  Here, T is the type of the State, which is
 at our discretion.  In this case, it is the Store struct, which contains the database.
   The method_router is the router that we want to merge into the main router.

   The trait bounds here can be a bit confusing.  Send + Sync exist because we're in Tokioland
   where Axum may be running this on multiple threads.  The static is a bit different.

   In Rust, this essentially means that whatever T is, has to live for the entire lifetime of the program.
   This is necessary if our T has references to other data inside of it.  Rust needs to be able
   to guarantee that the data will be there for as long as this Router holds onto it.

   So this 'static lifetime is telling Rust that T can either contain no references at all,
   or they must be references that live for the life of the program (ie static themselves)

   We need it here not only because Axum's handlers require it, but also because we can tell that
   Router will exist for the life of the program an also capture the state of T.  So if T is not static,
   then we have a problem.

   Our Store, luckily, passes this test fine.  If you take a look, you'll now be able to see
   why those Arc<Mutex> were necessary!

*/
pub fn merged_route<T>(path: &str, method_router: MethodRouter<T>) -> Router<T>
where
    T: Clone + Send + Sync + 'static,
{
    Router::new().route(path, method_router)
}

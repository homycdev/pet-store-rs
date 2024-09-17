use chrono::{DateTime, Utc};
use pet::PetCategory;
use pet::PetSize;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::pet;

#[derive(Serialize, Deserialize, sqlx::Type, PartialEq, Debug, Clone)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum OrderStatus {
    Awaiting,
    Approved,
    Delivered,
    Cancelled,
}

#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug, Clone)]
pub struct Order {
    id: u64,
    user_id: u64,
    pet_id: u64,
    quantity: u64,
    ship_date: Option<DateTime<Utc>>,
    status: OrderStatus,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct OrderPet {
    category: PetCategory,
    pet_size: PetSize,
}
mod service {
    use axum::{debug_handler, routing::get, Router};
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::IntoResponse,
        Json,
    };

    use crate::{error::AppError, AppState};
    use anyhow::Result;

    use super::{
        storage::{self, OrderDB},
        Order,
    };
    #[debug_handler]
    pub async fn get_order(state: State<AppState>, Path(order_id): Path<u64>) -> impl IntoResponse {
        match storage::get(state.0.clone(), order_id).await {
            Ok(Some(order)) => (StatusCode::OK, Json::<Order>(order.into())).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND, Json(())).into_response(),
            Err(err) => AppError(err).into_response(),
        }
    }

    pub async fn delete(state: State<AppState>, Path(order_id): Path<u64>) -> impl IntoResponse {
        match storage::delete(state.0.clone(), order_id).await {
            Ok(_) => (StatusCode::OK, Json(())).into_response(),
            Err(err) => AppError(err).into_response(),
        }
    }

    pub async fn list_orders(
        state: State<AppState>,
        Path(user_id): Path<u64>,
    ) -> impl IntoResponse {
        match storage::list(state.0.clone(), user_id).await {
            Ok(res) => (
                StatusCode::OK,
                Json::<Vec<Order>>(res.into_iter().map(|o| o.into()).collect()),
            )
                .into_response(),

            Err(err) => AppError(err).into_response(),
        }
        // let orders = storage::list(state, user_id).await?;
        // Ok(orders.into_iter().map(|o| o.into()).collect())
    }
    pub async fn create_order(
        state: State<AppState>,
        Json(order): Json<Order>,
    ) -> impl IntoResponse {
        let order_db = OrderDB {
            id: order.id as i64,
            user_id: order.user_id as i64,
            pet_id: order.pet_id as i64,
            quantity: order.quantity as i64,
            ship_date: order.ship_date,
            status: order.status,
        };
        match storage::create(state.0.clone(), order_db)
            .await
            .map_err(|err| AppError(err))
        {
            Ok(_) => (StatusCode::OK, Json(())).into_response(),
            Err(err) => err.into_response(),
        }
    }

    pub async fn update_order(
        state: State<AppState>,
        Json(order): Json<Order>,
    ) -> impl IntoResponse {
        let order_db = OrderDB {
            id: order.id as i64,
            user_id: order.user_id as i64,
            pet_id: order.pet_id as i64,
            quantity: order.quantity as i64,
            ship_date: order.ship_date,
            status: order.status,
        };
        match storage::update(state.0.clone(), order_db).await {
            Ok(_) => (StatusCode::OK, Json(())).into_response(),
            Err(err) => AppError(err).into_response(),
        }
    }
}
mod storage {

    use axum::extract::State;
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, Postgres};

    use crate::{persistence::ArcPgPool, user, AppState};

    use super::{Order, OrderStatus};
    use anyhow::Result;

    pub struct OrderStorage {
        pub storage: ArcPgPool,
    }

    #[derive(Clone, FromRow, PartialEq, Debug)]
    pub struct OrderDB {
        pub id: i64,
        pub pet_id: i64,
        pub user_id: i64,
        pub quantity: i64,
        pub ship_date: Option<DateTime<Utc>>,
        pub status: OrderStatus,
    }

    impl Into<Order> for OrderDB {
        fn into(self) -> Order {
            Order {
                id: self.id as u64,
                user_id: self.user_id as u64,
                pet_id: self.pet_id as u64,
                quantity: self.quantity as u64,
                ship_date: self.ship_date,
                status: self.status,
            }
        }
    }

    #[tracing::instrument(skip(state))]
    pub async fn create(state: AppState, order: OrderDB) -> Result<()> {
        sqlx::query::<Postgres>("insert into orders ( id, pet_id, user_id, quantity, ship_date, status) values ($1,$2,$3,$4,$5,$6);")
            .bind(order.id)
            .bind(order.pet_id)
            .bind(order.user_id)
            .bind(order.quantity)
            .bind(order.ship_date)
            .bind(order.status)
            .execute(&state.db.clone())
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(state))]
    pub async fn get(state: AppState, order_id: u64) -> Result<Option<OrderDB>> {
        let res: Option<OrderDB> = sqlx::query_as(
            "select * 
            from orders o 
            where o.id = $1",
        )
        .bind(order_id as i64)
        .fetch_optional(&state.db.clone())
        .await?;

        Ok(res)
    }

    #[tracing::instrument(skip(state))]
    pub async fn list(state: AppState, user_id: u64) -> Result<Vec<OrderDB>> {
        let res: Vec<OrderDB> = sqlx::query_as(
            "select * 
            from orders o 
            where o.user_id = $1
            order by o.id desc",
        )
        .bind(user_id as i64)
        .fetch_all(&state.db.clone())
        .await?;

        Ok(res)
    }

    #[tracing::instrument(skip(state))]
    pub async fn delete(state: AppState, id: u64) -> Result<()> {
        let _ = sqlx::query("delete from orders where id = $1")
            .bind(id as i64)
            .execute(&state.db.clone())
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(state))]
    pub async fn update(state: AppState, o: OrderDB) -> Result<()> {
        let _ = sqlx::query::<Postgres>(
            "insert into orders as o(id, pet_id, user_id, quantity, ship_date, status) 
            values ($1, $2, $3, $4, $5, $6)
            on conflict(id) 
            do update set 
                pet_id = excluded.pet_id, 
                user_id = excluded.user_id, 
                quantity = excluded.quantity, 
                ship_date = excluded.ship_date, 
                status = excluded.status;
            ",
        )
        .bind(o.id)
        .bind(o.pet_id)
        .bind(o.user_id)
        .bind(o.quantity)
        .bind(o.ship_date)
        .bind(o.status)
        .execute(&state.db.clone())
        .await?;

        Ok(())
    }
}

pub mod api {

    use axum::{
        extract::State,
        routing::{delete, get, post, put},
        Router,
    };
    use storage::OrderStorage;

    use crate::AppState;

    use super::service;
    use super::storage;

    pub(crate) fn create_router() -> Router<AppState> {
        Router::new()
            .route("/", post(service::create_order))
            .route(
                "/:order_id",
                get(service::get_order)
                    .delete(service::delete)
                    .post(service::update_order),
            )
            .route("/all/:user_id", get(service::list_orders))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        persistence::{Storage, StorageConfig, StorageIml},
        AppStateInner,
    };
    use axum::{extract::State, serve::Serve, Router};

    use crate::{config::AppConfig, AppState};
    use anyhow::Result;

    async fn fixture() -> Result<State<AppState>> {
        let config = AppConfig::load_config()?;

        let storage_conf = StorageConfig {
            db_path: config.db(),
        };
        let storage = StorageIml.conn(storage_conf).await?;
        StorageIml.migrate(storage.clone()).await?;

        Ok(State(AppState {
            inner: Arc::new(AppStateInner {
                db: storage,
                version: "0.0.1".to_string(),
            }),
        }))
    }

    async fn create_server(
        routes: Router<AppState>,
        state: AppState,
    ) -> Result<Serve<Router, Router>> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:9009").await?;
        let router = routes.with_state(state.clone());
        let server = axum::serve(listener, router);
        Ok(server)
    }
    // mod api {
    //     use std::{net::IpAddr, time::Duration};

    //     use axum::Router;
    //     use reqwest::{Response, StatusCode};
    //     use serde_json::json;

    //     use crate::{
    //         orders::{api, tests::create_server, Order, OrderStatus},
    //         AppState,
    //     };

    //     use super::fixture;

    //     #[tokio::test]
    //     async fn create_order_test() -> anyhow::Result<()> {
    //         let state = fixture().await?;
    //         let router: Router<AppState> = api::create_router().with_state(state.0.clone());
    //         let order: Order = Order {
    //             id: 0,
    //             user_id: 0,
    //             pet_id: 0,
    //             quantity: 0,
    //             ship_date: None,
    //             status: OrderStatus::Awaiting,
    //         };

    //         let req_body = serde_json::to_string(&order)?;

    //         let _ = create_server(router, state.0.clone()).await?;
    //         let client = reqwest::Client::builder()
    //             .connection_verbose(true)
    //             .connect_timeout(Duration::from_millis(300))
    //             .build()?;
            

    //         // .get("http://localhost:9009/1")
    //         // .send()
    //         // .await?;

    //         // let res = &client.get("https://hyper.rs").send().await?;
    //         // let res = res.clone();

    //         // assert!(&res.status().is_success());

    //         // assert!(res.is_ok());
    //         // assert!(res.is_ok());
    //         // let res = reqwest::Client::new()
    //         // .get("http://localhost:9009/1")
    //         // .send()
    //         // .await?;

    //         // let res = reqwest::Client::new()
    //         //     .post("http://127.0.0.1:9003/order/create")
    //         //     .body(req_body)
    //         //     .send()
    //         //     .await?;

    //         // assert_eq!(&res.status().as_str(), &"200");

    //         // let body = serde_json::from_str::<Order>(&res.text().await?)?;

    //         // assert_eq!(body, order);

    //         state.0.shutdown().await?;
    //         Ok(())
    //     }
    // }
    mod storage {

        use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

        use crate::orders::{
            storage::{self, OrderDB},
            tests::fixture,
            OrderStatus,
        };

        #[tokio::test]
        async fn get_order() -> anyhow::Result<()> {
            let state = fixture().await?;
            // let state = Arc::new(state);

            let res = storage::get(state.0.clone(), 0).await?;

            assert_eq!(None, res);

            state.0.shutdown().await?;

            Ok(())
        }
        #[tokio::test]
        async fn insert_order() -> anyhow::Result<()> {
            let state = fixture().await?;

            let d = NaiveDate::from_ymd_opt(2024, 8, 14).unwrap();
            let t = NaiveTime::from_hms_milli_opt(23, 22, 0, 0).unwrap();
            let test_order = OrderDB {
                id: 1,
                pet_id: 0,
                user_id: 0,
                quantity: 32,
                ship_date: Some(NaiveDateTime::new(d, t).and_utc()),
                status: OrderStatus::Approved,
            };
            storage::create(state.0.clone(), test_order.clone()).await?;
            // orders_storage.create(test_order.clone()).await?;
            let get_res = storage::get(state.0.clone(), 1).await?;

            assert_eq!(test_order.id, get_res.unwrap().id);

            storage::delete(state.0.clone(), 1).await?;

            state.0.shutdown().await?;

            Ok(())
        }
        #[tokio::test]
        async fn update_order() -> anyhow::Result<()> {
            let state = fixture().await?;
            let d = NaiveDate::from_ymd_opt(2024, 8, 14).unwrap();
            let t = NaiveTime::from_hms_milli_opt(23, 22, 0, 0).unwrap();

            let test_order = OrderDB {
                id: 1,
                pet_id: 0,
                user_id: 0,
                quantity: 32,
                ship_date: Some(NaiveDateTime::new(d, t).and_utc()),
                status: OrderStatus::Approved,
            };

            storage::create(state.0.clone(), test_order).await?;
            storage::update(
                state.0.clone(),
                OrderDB {
                    id: 1,
                    pet_id: 0,
                    user_id: 1,
                    quantity: 0,
                    ship_date: Some(NaiveDateTime::new(d, t).and_utc()),
                    status: OrderStatus::Cancelled,
                },
            )
            .await?;

            let result = storage::get(state.0.clone(), 1).await?;

            assert_eq!(
                Some(OrderDB {
                    id: 1,
                    pet_id: 0,
                    user_id: 1,
                    quantity: 0,
                    ship_date: Some(NaiveDateTime::new(d, t).and_utc()),
                    status: OrderStatus::Cancelled,
                }),
                result
            );

            storage::delete(state.0.clone(), 1).await?;
            state.0.shutdown().await?;

            Ok(())
        }
    }
}

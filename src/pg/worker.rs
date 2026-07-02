use std::time::Instant;

use tokio::sync::mpsc;
use tokio_postgres::{Client, NoTls, Row};

use crate::pg::{CatalogNode, PgCommand, PgEvent, ResultColumn};

pub(crate) async fn pg_worker(
    mut commands_rx: mpsc::UnboundedReceiver<PgCommand>,
    events_tx: mpsc::UnboundedSender<PgEvent>,
) {
    let mut client: Option<Client> = None;

    while let Some(command) = commands_rx.recv().await {
        match command {
            PgCommand::Connect(profile) => {
                let connect_result =
                    tokio_postgres::connect(&profile.connection_string(), NoTls).await;

                match connect_result {
                    Ok((pg_client, connection)) => {
                        tokio::spawn(async move {
                            if let Err(error) = connection.await {
                                eprintln!("postgres connection failed: {error}");
                            }
                        });

                        client = Some(pg_client);
                        let _ = events_tx.send(PgEvent::Connected {
                            profile_id: profile.id.clone(),
                        });
                        load_catalog(client.as_ref(), &events_tx).await;
                    }
                    Err(error) => {
                        let _ = events_tx.send(PgEvent::QueryFailed(error.to_string()));
                    }
                }
            }
            PgCommand::Execute { sql } => {
                let Some(pg_client) = client.as_ref() else {
                    let _ = events_tx.send(PgEvent::QueryFailed(
                        "Not connected. Select local-dev first.".into(),
                    ));
                    continue;
                };

                if let Err(error) = pg_query::parse(&sql) {
                    let _ = events_tx.send(PgEvent::QueryFailed(error.to_string()));
                    continue;
                }

                let _ = events_tx.send(PgEvent::QueryStarted);
                let started = Instant::now();

                match pg_client.query(sql.as_str(), &[]).await {
                    Ok(rows) => {
                        let elapsed_ms = started.elapsed().as_millis();
                        let columns = rows
                            .first()
                            .map(|row| {
                                row.columns()
                                    .iter()
                                    .map(|column| {
                                        ResultColumn::new(column.name(), column.type_().name())
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        let rendered_rows = rows.iter().map(render_row).collect::<Vec<_>>();
                        let row_count = rendered_rows.len();

                        let _ = events_tx.send(PgEvent::QueryResult {
                            columns,
                            rows: rendered_rows,
                            elapsed_ms,
                        });
                        let _ = events_tx.send(PgEvent::QueryCompleted {
                            rows: row_count,
                            elapsed_ms,
                        });
                    }
                    Err(error) => {
                        let _ = events_tx.send(PgEvent::QueryFailed(error.to_string()));
                    }
                }
            }
            PgCommand::Cancel => {
                let _ = events_tx.send(PgEvent::Notice("cancel requested".into()));
            }
            PgCommand::LoadCatalogNode { .. } => {
                load_catalog(client.as_ref(), &events_tx).await;
            }
        }
    }
}

async fn load_catalog(client: Option<&Client>, events_tx: &mpsc::UnboundedSender<PgEvent>) {
    let Some(client) = client else {
        return;
    };

    let sql = "\
        select schemaname, tablename \
        from pg_catalog.pg_tables \
        where schemaname not in ('pg_catalog', 'information_schema') \
        order by schemaname, tablename \
        limit 64";

    match client.query(sql, &[]).await {
        Ok(rows) => {
            let mut nodes = vec![
                CatalogNode::database("app_db"),
                CatalogNode::schema("public"),
                CatalogNode::folder("tables"),
            ];

            nodes.extend(rows.iter().filter_map(|row| {
                let table: Option<String> = row.try_get("tablename").ok();
                table.map(CatalogNode::table)
            }));

            let _ = events_tx.send(PgEvent::CatalogNodeLoaded {
                parent_id: "root".into(),
                nodes,
            });
        }
        Err(error) => {
            let _ = events_tx.send(PgEvent::QueryFailed(error.to_string()));
        }
    }
}

fn render_row(row: &Row) -> Vec<String> {
    row.columns()
        .iter()
        .enumerate()
        .map(|(ix, column)| render_cell(row, ix, column.type_().name()))
        .collect()
}

fn render_cell(row: &Row, ix: usize, pg_type: &str) -> String {
    match pg_type {
        "bool" => row
            .try_get::<usize, Option<bool>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "int2" => row
            .try_get::<usize, Option<i16>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "int4" => row
            .try_get::<usize, Option<i32>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "int8" => row
            .try_get::<usize, Option<i64>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "float4" => row
            .try_get::<usize, Option<f32>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "float8" => row
            .try_get::<usize, Option<f64>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "json" | "jsonb" => row
            .try_get::<usize, Option<serde_json::Value>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        _ => row
            .try_get::<usize, Option<String>>(ix)
            .ok()
            .flatten()
            .unwrap_or_else(|| "NULL".into()),
    }
}

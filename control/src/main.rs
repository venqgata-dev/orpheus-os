use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};

use rusqlite::{Connection, params};
use serde::Deserialize;

use std::{fs, sync::Arc};

use tokio::{
    net::TcpListener,
    sync::Mutex,
    time::{sleep, Duration},
};

use chrono::Utc;

#[derive(Debug, Deserialize)]
struct NodeList {
    nodes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ChainHeadResponse {
    node_id: String,
    chain_head: String,
}

struct AppState {
    db: Mutex<Connection>,
}

fn load_nodes() -> Vec<String> {
    let content = fs::read_to_string("nodes.json")
        .expect("nodes.json missing");

    let list: NodeList =
        serde_json::from_str(&content)
        .expect("invalid nodes.json");

    list.nodes
}

fn init_db() -> Connection {

    let conn = Connection::open("control.db").unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS node_state(
            node_id TEXT PRIMARY KEY,
            chain_head TEXT,
            last_seen INTEGER,
            status TEXT
        )",
        [],
    ).unwrap();

    conn
}

async fn dashboard(
    State(state): State<Arc<AppState>>
) -> Html<String> {

    let conn = state.db.lock().await;

    let mut stmt = conn.prepare(
        "SELECT node_id, chain_head, status FROM node_state"
    ).unwrap();

    let rows = stmt.query_map([], |row| {

        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))

    }).unwrap();

    let mut html_rows = String::new();

    for row in rows {

        let (node, head, status) = row.unwrap();

        let class = if status == "ONLINE" {
            "online"
        } else {
            "offline"
        };

        html_rows.push_str(&format!(
            "<tr>
                <td>{}</td>
                <td>{}</td>
                <td class=\"{}\">{}</td>
            </tr>",
            node,
            &head[..12],
            class,
            status
        ));
    }

    let page = format!(
        "<html>
        <head>

        <title>Orpheus Control Plane</title>

        <meta http-equiv='refresh' content='3'>

        <style>

        body {{
            font-family: Arial;
            background:#0f172a;
            color:white;
            padding:40px;
        }}

        table {{
            border-collapse: collapse;
            width:100%;
        }}

        th, td {{
            padding:12px;
            border:1px solid #334155;
        }}

        th {{
            background:#1e293b;
        }}

        .online {{
            color:#22c55e;
            font-weight:bold;
        }}

        .offline {{
            color:#ef4444;
            font-weight:bold;
        }}

        </style>

        </head>

        <body>

        <h1>Orpheus Control Plane</h1>

        <table>

        <tr>
            <th>Node</th>
            <th>Chain Head</th>
            <th>Status</th>
        </tr>

        {}

        </table>

        </body>
        </html>",
        html_rows
    );

    Html(page)
}

async fn monitor_loop(state: Arc<AppState>) {

    loop {

        let nodes = load_nodes();

        let conn = state.db.lock().await;

        conn.execute(
            "UPDATE node_state
             SET status='OFFLINE'
             WHERE strftime('%s','now') - last_seen > 15",
            [],
        ).unwrap();

        drop(conn);

        for node in nodes {

            let url = format!("{}/audit/chain-head", node);

            if let Ok(resp) = reqwest::get(&url).await {

                if let Ok(data) =
                    resp.json::<ChainHeadResponse>().await {

                    let conn = state.db.lock().await;

                    conn.execute(
                        "INSERT OR REPLACE INTO node_state
                        (node_id,chain_head,last_seen,status)
                        VALUES (?1,?2,?3,'ONLINE')",
                        params![
                            data.node_id,
                            data.chain_head,
                            Utc::now().timestamp()
                        ],
                    ).unwrap();

                }

            }

        }

        sleep(Duration::from_secs(5)).await;

    }

}

#[tokio::main(flavor = "current_thread")]
async fn main() {

    println!("Orpheus Controller starting...");

    let conn = init_db();

    let state = Arc::new(AppState {
        db: Mutex::new(conn),
    });

    let monitor_state = state.clone();

    tokio::spawn(async move {
        monitor_loop(monitor_state).await;
    });

    let app = Router::new()
        .route("/", get(dashboard))
        .with_state(state);

    let listener =
        TcpListener::bind("127.0.0.1:9000")
        .await
        .unwrap();

    println!("Dashboard running at http://127.0.0.1:9000");

    axum::serve(listener, app)
        .await
        .unwrap();
}

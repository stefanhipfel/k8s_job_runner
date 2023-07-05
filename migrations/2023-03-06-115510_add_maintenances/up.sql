CREATE TABLE maintenances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid VARCHAR UNIQUE NOT NULL,
    name VARCHAR,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME,
    failed_attempts INT NOT NULL,
    status VARCHAR NOT NULL,
    scheduled_for DATETIME,
    downtime_window_start DATETIME,
    downtime_window_end DATETIME,
    job_id INTEGER REFERENCES jobs(id) NOT NULL
)
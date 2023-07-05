CREATE TABLE jobs (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    version VARCHAR,
    docker_image VARCHAR NOT NULL,
    docker_image_tag VARCHAR NOT NULL
)
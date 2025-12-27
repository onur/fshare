# fshare

S3 compatible, stateless temporary file share service.

## Usage

```sh
docker run -d \
    --name fshare \
    -e AWS_REGION=us-east-1 \
    -e AWS_ACCESS_KEY_ID=admin \
    -e AWS_SECRET_ACCESS_KEY=12345678 \
    -e AWS_BUCKET=test \
    -p 8080:8080 \
    ghcr.io/onur/fshare:latest
```

Or you can use it with local running S3 object storage with rustfs, from provided [docker-compose.yml](docker-compose.yml) file:

```sh
wget https://raw.githubusercontent.com/onur/fshare/refs/heads/master/docker-compose.yml
docker compose up -d
```

Visit http://localhost:9001 to create a bucket. Then you can access fshare from http://localhost:9000.

**You need to create a lifecycle policy to remove old files.**

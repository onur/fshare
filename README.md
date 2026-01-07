# fshare

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

S3 compatible, stateless temporary file share service built with Rust.

## Features

- **Stateless & Lightweight:** Built with Axum and the AWS SDK for Rust.
- **S3 Compatible:** Works with AWS S3, MinIO, [rustfs](https://github.com/rustfs/rustfs), and other S3-compatible storage.
- **Terminal Friendly:** Detects non-browser clients (like `curl`) and returns plain text URLs.
- **Multipart Uploads:** Efficiently handles large file uploads.
- **Configurable Expirations:** Users can choose from a list of predefined durations.
- **Self-contained:** Templates are embedded into the binary.

## Usage

### Docker

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

### Docker Compose

You can use it with local running S3 object storage with [rustfs](https://github.com/rustfs/rustfs), using the provided [docker-compose.yml](docker-compose.yml):

```sh
wget https://raw.githubusercontent.com/onur/fshare/refs/heads/master/docker-compose.yml
docker compose up -d
```

Visit `http://localhost:9001` to create a bucket. Then you can access fshare from `http://localhost:8080`.

### Terminal (curl)

Upload a file directly from your terminal:

```sh
curl -F "file=@photo.jpg" http://localhost:8080
```

Specify expiration (in minutes):

```sh
curl -F "file=@photo.jpg" -F "expiration=60" http://localhost:8080
```

## Configuration

fshare uses the official AWS SDK for Rust. You can use standard [AWS environment variables](https://docs.aws.amazon.com/sdkref/latest/guide/environment-variables.html) to configure the backend.

| Environment Variable | Description | Default |
|----------------------|-------------|---------|
| `AWS_BUCKET` | **Required**. S3 bucket name. | |
| `MAX_UPLOAD_SIZE` | Maximum upload size in megabytes. | `10` |
| `ALLOWED_DURATIONS` | Comma separated list of durations in minutes. | `30,60,360,1440,10080` |
| `ID_LENGTH` | Length of the generated upload IDs. | `8` |
| `SOCKET_ADDR` | Socket address to bind the server. | `0.0.0.0:8080` |

## Deployment

### AWS Lambda

fshare can be deployed to AWS Lambda using the [AWS Lambda Web Adapter](https://github.com/awslabs/aws-lambda-web-adapter). A sample Terraform configuration is provided in the [terraform/aws-lambda](terraform/aws-lambda) directory.

## Maintenance

**IMPORTANT:** You must create a [Lifecycle Policy](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lifecycle-mgmt.html) for your S3 bucket to automatically remove expired files. fshare marks objects with an expiration date, but S3 requires a policy to perform the actual deletion.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

FROM alpine:latest

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the binary from the builder image
COPY ./target/release/actix-kafka-redis /app/actix-kafka-redis

# Expose the port that the application will run on
EXPOSE 8080

# Set the entrypoint command to run the application
CMD ["./actix-kafka-redis"]
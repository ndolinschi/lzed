Extractors pull data out of a request. `Json<T>` deserializes the body into any `T: Deserialize`, and returning `Json<U>` serializes `U: Serialize` back to the client.

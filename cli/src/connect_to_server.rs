fn connect_to_server(host: String, port: int32) {
    TcpStream::connect(host)
}


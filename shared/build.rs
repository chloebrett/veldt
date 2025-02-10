fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/echo.proto", "proto/pmodel.proto"], &["proto"])
        .unwrap();
}

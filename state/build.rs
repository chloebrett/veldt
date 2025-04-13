fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["proto/action_proto.proto", "proto/broadcast_actions.proto"],
            &["proto"],
        )
        .unwrap();
}

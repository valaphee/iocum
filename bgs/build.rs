use std::path::PathBuf;

fn main() {
    prost_build::Config::new()
        .file_descriptor_set_path(
            PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("file_descriptor_set.bin"),
        )
        .service_generator(Box::new(ServiceGenerator))
        .compile_protos(
            &glob::glob("src/**/*.proto")
                .unwrap()
                .map(|path| path.unwrap())
                .collect::<Vec<_>>(),
            &["src/"],
        )
        .unwrap();
}

struct ServiceGenerator;

impl prost_build::ServiceGenerator for ServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        buf.push_str("#[async_trait::async_trait]");
        buf.push_str(&format!("pub trait {} {{", service.name));
        for method in service.methods.iter() {
            if method.output_proto_type == ".bgs.protocol.NO_RESPONSE" {
                buf.push_str(&format!("    async fn {}(&self, header: crate::bgs::protocol::Header, request: {});", method.name, method.input_type));
            } else {
                buf.push_str(&format!("    async fn {}(&self, header: crate::bgs::protocol::Header, request: {}) -> (crate::bgs::protocol::Header, {});", method.name, method.input_type, method.output_type));
            }
        }
        buf.push_str("}");

        buf.push_str("#[async_trait::async_trait]");
        buf.push_str(&format!("impl {} for crate::bgs::RemoteService {{", service.name));
        for method in service.methods.iter() {
            if method.output_proto_type == ".bgs.protocol.NO_RESPONSE" {
                buf.push_str(&format!("    async fn {}(&self, header: crate::bgs::protocol::Header, request: {}) {{", method.name, method.input_type));
                buf.push_str("        self.rpc_no_response(header, request).await")
            } else {
                buf.push_str(&format!("    async fn {}(&self, header: crate::bgs::protocol::Header, request: {}) -> (crate::bgs::protocol::Header, {}) {{", method.name, method.input_type, method.output_type));
                buf.push_str("        self.rpc(header, request).await")
            }
            buf.push_str("    }")
        }
        buf.push_str("}");
    }
}

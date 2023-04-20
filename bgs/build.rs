use std::{collections::HashMap, path::PathBuf};

use prost::Message;

fn main() {
    prost_build::Config::new()
        .file_descriptor_set_path(
            PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("file_descriptor_set.bin"),
        )
        .service_generator(Box::new(ServiceGenerator::default()))
        .compile_protos(
            &glob::glob("src/**/*.proto")
                .unwrap()
                .map(|path| path.unwrap())
                .collect::<Vec<_>>(),
            &["src/"],
        )
        .unwrap();
}

#[derive(Default)]
struct ServiceGenerator(HashMap<String, (u32, HashMap<String, u32>)>);

impl ServiceGenerator {
    fn initialize(&mut self) {
        if self.0.is_empty() {
            let file_descriptor_set = std::fs::read(
                PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("file_descriptor_set.bin"),
            )
            .unwrap();
            let file_descriptor_set =
                FileDescriptorSet::decode(file_descriptor_set.as_slice()).unwrap();
            for file_descriptor_proto in file_descriptor_set.file {
                let package = file_descriptor_proto.package.unwrap();
                for service_descriptor_proto in file_descriptor_proto.service {
                    let mut methods = HashMap::new();
                    for method_descriptor_proto in service_descriptor_proto.method {
                        methods.insert(
                            method_descriptor_proto.name.unwrap(),
                            method_descriptor_proto
                                .options
                                .unwrap()
                                .method_options
                                .unwrap()
                                .id
                                .unwrap(),
                        );
                    }
                    self.0.insert(
                        format!("{}.{}", package, service_descriptor_proto.name.unwrap()),
                        (
                            hashers::fnv::fnv1a32(
                                service_descriptor_proto
                                    .options
                                    .unwrap()
                                    .service_options
                                    .unwrap()
                                    .descriptor_name
                                    .unwrap()
                                    .as_bytes(),
                            ) as u32,
                            methods,
                        ),
                    );
                }
            }
        }
    }
}

impl prost_build::ServiceGenerator for ServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        self.initialize();

        let (service_hash, methods) = self
            .0
            .get(&format!("{}.{}", service.package, service.proto_name))
            .unwrap();
        buf.push_str("#[async_trait::async_trait]");
        buf.push_str(&format!("pub trait {} {{", service.name));
        for method in service.methods.iter() {
            if method.output_proto_type == ".bgs.protocol.NO_RESPONSE" {
                buf.push_str(&format!(
                    "    async fn {}(&self, request: {});",
                    method.name, method.input_type
                ));
            } else {
                buf.push_str(&format!(
                    "    async fn {}(&self, request: {}) -> {};",
                    method.name, method.input_type, method.output_type
                ));
            }
        }
        buf.push_str("}");

        buf.push_str("#[async_trait::async_trait]");
        buf.push_str(&format!(
            "impl {} for crate::bgs::RemoteService {{",
            service.name
        ));
        for method in service.methods.iter() {
            if method.output_proto_type == ".bgs.protocol.NO_RESPONSE" {
                buf.push_str(&format!(
                    "    async fn {}(&self, request: {}) {{",
                    method.name, method.input_type
                ));
                buf.push_str(&format!(
                    "        println!(\"{}::{}({{:?}})\", request);",
                    service.name, method.name
                ));
                buf.push_str(&format!(
                    "        self.request_no_response({}, {}, request);",
                    service_hash,
                    methods.get(&method.proto_name).unwrap()
                ));
            } else {
                buf.push_str(&format!(
                    "    async fn {}(&self, request: {}) -> {} {{",
                    method.name, method.input_type, method.output_type
                ));
                buf.push_str(&format!(
                    "        println!(\"{}::{}({{:?}})\", request);",
                    service.name, method.name
                ));
                buf.push_str(&format!(
                    "        let response = self.request({}, {}, request).await;",
                    service_hash,
                    methods.get(&method.proto_name).unwrap()
                ));
                buf.push_str("        println!(\"{:?}\", response);");
                buf.push_str("        response");
            }
            buf.push_str("    }")
        }
        buf.push_str("}");
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorSet {
    #[prost(message, repeated, tag = "1")]
    pub file: ::prost::alloc::vec::Vec<FileDescriptorProto>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorProto {
    #[prost(string, optional, tag = "2")]
    pub package: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "6")]
    pub service: ::prost::alloc::vec::Vec<ServiceDescriptorProto>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServiceDescriptorProto {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "2")]
    pub method: ::prost::alloc::vec::Vec<MethodDescriptorProto>,
    #[prost(message, optional, tag = "3")]
    pub options: ::core::option::Option<ServiceOptions>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServiceOptions {
    #[prost(message, optional, tag = "90000")]
    pub service_options: ::core::option::Option<bgs::protocol::BgsServiceOptions>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MethodDescriptorProto {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "4")]
    pub options: ::core::option::Option<MethodOptions>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MethodOptions {
    #[prost(message, optional, tag = "90000")]
    pub method_options: ::core::option::Option<bgs::protocol::BgsMethodOptions>,
}

pub mod bgs {
    pub mod protocol {
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct BgsServiceOptions {
            #[prost(string, optional, tag = "1")]
            pub descriptor_name: ::core::option::Option<::prost::alloc::string::String>,
        }

        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct BgsMethodOptions {
            #[prost(uint32, optional, tag = "1")]
            pub id: ::core::option::Option<u32>,
        }
    }
}

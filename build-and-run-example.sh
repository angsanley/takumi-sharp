#!/bin/bash

cargo build --release

cp target/release/liblibtakumi.so takumi-sharp/TakumiSharp.Example/bin/Debug/net10.0/libtakumi.so

dotnet run --project takumi-sharp/TakumiSharp.Example/TakumiSharp.Example.csproj
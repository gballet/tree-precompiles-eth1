#!/usr/bin/env ruby

require 'erb'

GoPath = ENV["GOPATH"]

raise if GoPath.length == 0

system("cargo build --target=wasm32-unknown-unknown --release")

template = ERB.new("// Copyright 2020 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package vm

var wasm<%= precompile.capitalize %> = []byte{<%= datastr %>}")

["verify", "update"].each do |precompile|
  datastr = File.read("#{ENV["PWD"]}/target/wasm32-unknown-unknown/release/precompile_1x_tree_#{precompile}.wasm").bytes.inspect.gsub(/\[|\]/, "")
  File.write("#{GoPath}/src/github.com/ethereum/go-ethereum/core/vm/wasm_#{precompile}.go", template.result(binding))
end

#!/usr/bin/env ruby
# frozen_string_literal: true

require 'erb'
require 'shellwords'

raise 'must provide a library base path' if ARGV[0].nil?
raise 'must provide a library to import' if ARGV[1].nil?
raise 'must provide an output directory' if ARGV[2].nil?

base = ARGV[0]
package = ARGV[1]
out_file = ARGV[2]
sources = ARGV[3].to_s.split(',').map do |source|
  source.gsub(%r{^.*#{Regexp.escape(base)}[/\\]?}, '').gsub(/\.rb$/, '')
end
auto_import_dir = File.dirname(__FILE__)
# Import the Ruby 2.6.3 sources.
command = [
  'ruby',
  '--disable-did_you_mean',
  '--disable-gems',
  "#{auto_import_dir}/get_constants_loaded.rb",
  base,
  package
]
constants = `#{command.shelljoin}`.split("\n").map { |const| const.split(',') }

# Add Rust glue, like this example for ostruct. Make a commit here.
template = File.read("#{auto_import_dir}/rust_glue.rs.erb")
renderer = ERB.new(template)
output = renderer.result(binding)
File.write(out_file.to_s, output)

# maws-menu

A simple console menu for the [Mozilla AWS CLI](https://github.com/mozilla-iam/mozilla-aws-cli).

This tool shows an interactive menu to select the account and role to use, and then calls maws with the selected role ARN. This avoids a round trip from the terminal to the browser and back. The tool remembers the last selections.

## Installation and Usage

1. Install this binary with `cargo install --git https://github.com/smarnach/maws-menu`.
1. Get the JSON file with accounts and roles from the `/api/roles` endpoint of the maws web interface and store it in `~/.config/maws/roles.json`.
1. Run the tool with `$(maws-menu)`

## Known Issues

This should have been implemented as a part of the Mozilla AWS CLI instead of a stand-alone tool. However, I felt like writing it in Rust. It doesn't make much sense, but it works just fine.

{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = with pkgs; [ rustup tmux openssl pkg-config gcc sqlite ];
}

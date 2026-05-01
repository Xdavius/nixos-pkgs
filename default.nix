{ pkgs ? import <nixpkgs> {} }:

rec {
  tachyfy = pkgs.callPackage ./pkgs/tachyfy {};
}

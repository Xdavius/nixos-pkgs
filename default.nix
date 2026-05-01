{ pkgs ? import <nixpkgs> {} }:

{
  tachyfy = pkgs.callPackage ./pkgs/tachyfy {};
}

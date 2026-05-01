{ pkgs ? import <nixpkgs> {} }:

let
  dir = ./pkgs;

  names = builtins.attrNames (builtins.readDir dir);

  isDir = name: (builtins.readDir dir)."${name}" == "directory";

  pkgsList = builtins.filter isDir names;
in
builtins.listToAttrs (map (name: {
  name = name;
  value = pkgs.callPackage (dir + "/${name}") {};
}) pkgsList)
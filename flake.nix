{
  outputs = { self, nixpkgs }:
  let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
  in {
    packages.x86_64-linux =
      import ./default.nix { inherit pkgs; };
  };
}

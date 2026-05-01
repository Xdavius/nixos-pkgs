# nixos-pkgs

Nixos PKGS AUR

Edit flake.nix

``` nix
inputs = {
    nixpkgs.follows = "glf-channels/nixpkgs";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Add xdaviuspkgs URL
    xdaviuspkgs.url = "github:Xdavius/nixos-pkgs";   
  };

  outputs =
    {
      nixpkgs,
      nixpkgs-unstable,
      self,
      # Add xdaviuspkgs output
      xdaviuspkgs,
      ...
    }:
let
      system = "x86_64-linux"; 

      # Configuration pour le nixpkgs stable (sera le 'pkgs' par défaut)
      pkgsStable = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };

      # Configuration pour le nixpkgs unstable (sera passé en argument spécial)
      pkgsUnstable = import nixpkgs-unstable {
        inherit system;
        config.allowUnfree = true;
      };
    in
    {
      nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      inherit system;
        pkgs = pkgsStable; 
        modules = [
          ./configuration.nix 
        ];

        specialArgs = {
          pkgs-unstable = pkgsUnstable;
          # Add xdaviuspkg argument
          inherit xdaviuspkgs;
        };
      };
    };
}
```

Add `daviuspkgs` to configuration.nix (or customConfig)
```nix
{
  lib,
  config,
  pkgs,
  pkgs-unstable,
  xdaviuspkgs,
  ...
}:
```

Add packages like this :

```nix
environment.systemPackages = [
  xdaviuspkgs.app.tachyfy
  ];
```

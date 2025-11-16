{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
    inputs.process-compose-flake.flakeModule
    inputs.cargo-doc-live.flakeModule
  ];
  perSystem = { config, self', pkgs, lib, ... }: {
    rust-project.crates."tap".crane.args = {
      src = lib.cleanSourceWith {
        src = ./../..;
        filter = path: type:
          let
            baseName = baseNameOf path;
          in
          # Exclude target, result, and .direnv directories
          baseName != "target" &&
          baseName != "result" &&
          baseName != "result-lib" &&
          baseName != ".direnv" &&
          baseName != "dobrich_sdd_500gb";
      };
      buildInputs = lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.darwin.apple_sdk.frameworks; [
          IOKit
        ]
      );
    };
    packages.default = self'.packages.tap;
  };
}

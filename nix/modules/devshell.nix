{ inputs, ... }:
{
  perSystem = { config, self', pkgs, lib, ... }: {
    devShells.default = pkgs.mkShell {
      name = "tap-shell";
      inputsFrom = [
        self'.devShells.rust
        config.pre-commit.devShell # See ./nix/modules/pre-commit.nix
      ];
      packages = with pkgs; [
        just
        nixd # Nix language server
        bacon
        ntfs3g # NTFS filesystem support
        dmraid # Intel RAID (ISW) management tool
        config.process-compose.cargo-doc-live.outputs.package
      ];
    };
  };
}

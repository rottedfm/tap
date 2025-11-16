{ ... }: {
  perSystem = { pkgs, ... }: {
    # Omnix configuration
  };

  flake = {
    # Define the om attribute that omnix expects
    om = {
      ci.default = {
        # Basic CI configuration for omnix
        dir = ".";
      };
    };
  };
}

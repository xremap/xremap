{
  inputs = {
    nci.url = "github:yusdacra/nix-cargo-integration";
  };
  outputs = inputs: inputs.nci.lib.makeOutputs {
    root = ./.;
  };
}
{ ... }:

{
  programs.git = {
    enable = true;
    lfs.enable = true;
    settings = {
      user = {
        name = "MCB-SMART-BOY";
        email = "mcb2720838051@gmail.com";
      };
      core = {
        editor = "hx";
        pager = "delta";
      };
      interactive.diffFilter = "delta --color-only";
      delta = {
        navigate = true;
        "side-by-side" = true;
      };
    };
  };
}

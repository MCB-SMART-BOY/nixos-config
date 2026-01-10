{ ... }:

{
  programs.git = {
    enable = true;
    lfs.enable = true;
    userName = "MCB-SMART-BOY";
    userEmail = "mcb2720838051@gmail.com";
    extraConfig = {
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

{ config, ... }:
let
  colours = config.colour-scheme.colours;
  theme = builtins.replaceStrings
    [
      "@surface@"
      "@on-surface@"
      "@on-surface-variant@"
      "@surface-container@"
      "@surface-container-high@"
      "@primary@"
      "@primary-container@"
      "@on-primary@"
      "@outline@"
      "@outline-variant@"
    ]
    [
      colours.surface
      colours.on-surface
      colours.on-surface-variant
      colours.surface-container
      colours.surface-container-high
      colours.primary
      colours.primary-container
      colours.on-primary
      colours.outline
      colours.outline-variant
    ]
    (builtins.readFile ./rounded.scss);
in
{
  xdg.configFile."tabsel/theme.scss".text = theme;
}

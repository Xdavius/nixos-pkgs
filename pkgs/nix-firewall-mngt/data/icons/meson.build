icon_sizes = ['16x16', '32x32', '48x48', '64x64', '128x128', '256x256', '512x512']

foreach size : icon_sizes
  install_data(
    'nix-firewall-mngt-' + size + '.png',
    install_dir: get_option('datadir') / 'icons' / 'hicolor' / size / 'apps',
    rename: 'org.glfos.nixfirewall.png',
  )
endforeach

# Scalable SVG icon
install_data(
  'nix-firewall-mngt.svg',
  install_dir: get_option('datadir') / 'icons' / 'hicolor' / 'scalable' / 'apps',
  rename: 'org.glfos.nixfirewall.svg',
)

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('bindir', get_option('prefix') / get_option('bindir'))

desktop_file = configure_file(
  input: 'org.glfos.nixfirewall.desktop.in',
  output: 'org.glfos.nixfirewall.desktop',
  configuration: desktop_conf,
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

# Polkit policy
polkit_conf = configuration_data()
polkit_conf.set('bindir', get_option('prefix') / get_option('bindir'))

configure_file(
  input: 'org.glfos.nixfirewall.in',
  output: 'org.glfos.nixfirewall.policy',
  configuration: polkit_conf,
  install: true,
  install_dir: get_option('datadir') / 'polkit-1' / 'actions',
)

# Icons
subdir('icons')

# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  # To compile HDHunter
  # config.vm.box = "generic/ubuntu2204"
  # config.vm.box_version = "4.3.12"
  # config.vm.synced_folder ".", "/vagrant"

  # To run HDHunter VM image
  config.vm.box = "hdhunter"

  config.vm.provider "virtualbox" do |vb|
    vb.memory = 16384
    vb.cpus = 8

    vb.customize ['modifyvm', :id, '--nested-hw-virt', 'on']
  end

  config.vm.provider "vmware_desktop" do |v|
    v.vmx["memsize"] = "16384"
    v.vmx["numvcpus"] = "8"

    v.vmx['vhv.enable'] = 'TRUE'
    v.vmx['monitor_control.restrict_backdoor'] = 'FALSE'
  end
end

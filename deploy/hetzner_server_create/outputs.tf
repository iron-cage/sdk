locals {
  host_server_ipv4 = hcloud_server.host_server.ipv4_address
}

# IPv4 address of the created GCE instance.
output "ipv4" {
  description = "The public IP address of hosted server."
  value       = local.host_server_ipv4
}

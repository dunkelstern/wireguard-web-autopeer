# WireGuard auto-peering

This is an additional service that may be run in parallel to the official WireGuard client
to aquire peer2peer functionality with WireGuard-web. Basically it uses the VPN connection
to the WireGuard-Web service to try to find peers in the same network as the requesting
client. If some are found they added as peers to the WireGuard tunnel to connect over the
local network instead of bouncing the data to the VPN server and back again.

This is called ZeroTrust Networking in the enterprise market, as services only are provided
over an encrypted VPN tunnel that can be attributed to a user, even if the person calling
the service sits physically on the same network as the service.

## Sketch of functionality:

1. Register tray item, two menu entries: Re-sync and Quit
2. Get network interfaces with default-net, save wireguard interfaces
3. Set up network change notifications with if-watch, on network change run from 4
4. Try to contact the default gateway on each wireguard interface with following info (JSON)
   - Pubkey
   - Standard gw-interface
   - IP+Mask on standard gw-interface
5. Server now checks if some of the other clients have
   - Same public IP
   - Same standard gw
   - Same Network/Mask on that interface
6. If matches are found a list of public-keys and IP addresses is returned
7. If the response contains any peers, add them to the wireguard interface with wireguard-control, if we're here
   from a previous iteration remove peers not in the response anymore from the wireguard interface
8. Remember which peers have been added
9. Periodically check if peers have changed networks by restarting the process on 4

## TODO

- Switch to `wireguard-uapi` as the current implementation fails to compile on osx and windows
- Auto-detect if we're running as root and skip systray integration
- Auto-detect if we're missing `CAP_NET_ADMIN=+eip` capability
- Remove peers we added on shutdown
- Shorter timeouts for communication
- Do not try to contact someone if there is no default gateway anymore, just remove peers
- on ifup wait for ip to arrive
- re-poll in intervals for changes 

## Running

1. To run the client it needs to have admin capabilities. Set with:
   ```bash
   sudo setcap CAP_NET_ADMIN=+eip target/release/wireguard-web-autopeer
   ```
2. If running as systemd you can add the caps there or run as admin.
3. If you want the icon displayed on Linux run:
   ```bash
   cp resources/wireguard-web-autopeer.svg $HOME/.local/share/icons
   ```
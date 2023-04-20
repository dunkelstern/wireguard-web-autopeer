use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};

use default_net::{self, ip::Ipv4Net, ip::Ipv6Net};

pub trait FirstIp {
    fn first_ip(&self) -> IpAddr;
}

impl FirstIp for Ipv4Net {
    fn first_ip(&self) -> IpAddr {
        if self.prefix_len == 32 {
            IpAddr::V4(self.addr)
        } else {
            let mask = u32::MAX << (32 - self.prefix_len);
            let mut ip: u32 = self.addr.into();
            ip = (ip & mask) + 1;
            IpAddr::V4(Ipv4Addr::from(ip))
        }
    }    
}

impl FirstIp for Ipv6Net {
    fn first_ip(&self) -> IpAddr {
        if self.prefix_len == 128 {
            IpAddr::V6(self.addr)
        } else {
            let mask = u128::MAX << (128 - self.prefix_len);
            let mut ip: u128 = self.addr.into();
            ip = (ip & mask) + 1;
            IpAddr::V6(Ipv6Addr::from(ip))
        }
    }
}

#[cfg(test)]
mod tests {
    use default_net::ip::{Ipv4Net, Ipv6Net};
    use std::net::IpAddr;

    use crate::ip_net::FirstIp;
   
    #[test]
    fn first_ip_in_net_v4() {
        assert_eq!(Ipv4Net::new("10.0.0.10".parse().unwrap(), 8).first_ip(), "10.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(Ipv4Net::new("10.1.0.10".parse().unwrap(), 8).first_ip(), "10.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(Ipv4Net::new("192.168.1.10".parse().unwrap(), 24).first_ip(), "192.168.1.1".parse::<IpAddr>().unwrap());
        assert_eq!(Ipv4Net::new("192.168.1.10".parse().unwrap(), 32).first_ip(), "192.168.1.10".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn first_ip_in_net_v6() {
        assert_eq!(Ipv6Net::new("fd12:3456:789a:1::".parse().unwrap(), 64).first_ip(), "fd12:3456:789a:1::1".parse::<IpAddr>().unwrap());
        assert_eq!(Ipv6Net::new("fd12:3456:789a:1::".parse().unwrap(), 8).first_ip(), "fd00::1".parse::<IpAddr>().unwrap());
    }

}
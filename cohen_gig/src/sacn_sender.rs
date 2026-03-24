use sacn::packet::{
    AcnRootLayerProtocol, DataPacketDmpLayer, DataPacketFramingLayer, E131RootLayer,
    E131RootLayerData, E131_DEFAULT_PRIORITY,
};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use uuid::Uuid;

const LOCALHOST_SACN_DESTINATION_PORT: u16 = 5568;

pub struct LocalhostSacnSender {
    socket: UdpSocket,
    cid: Uuid,
    name: String,
    sequences: HashMap<u16, u8>,
}

impl LocalhostSacnSender {
    pub fn new(name: &str) -> std::io::Result<Self> {
        let bind_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
        let socket = UdpSocket::bind(bind_addr)?;
        Ok(Self {
            socket,
            cid: Uuid::new_v4(),
            name: name.to_string(),
            sequences: HashMap::new(),
        })
    }

    pub fn send_property_values(
        &mut self,
        universe: u16,
        property_values: &[u8],
    ) -> std::io::Result<()> {
        let sequence = self.sequences.entry(universe).or_insert(0);
        let packet = build_data_packet(self.cid, &self.name, *sequence, universe, property_values);
        let dest = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), LOCALHOST_SACN_DESTINATION_PORT);
        self.socket.send_to(&packet.pack_alloc().unwrap(), dest)?;
        *sequence = sequence.wrapping_add(1);
        Ok(())
    }
}

fn build_data_packet(
    cid: Uuid,
    source_name: &str,
    sequence_number: u8,
    universe: u16,
    property_values: &[u8],
) -> AcnRootLayerProtocol<'static> {
    AcnRootLayerProtocol {
        pdu: E131RootLayer {
            cid,
            data: E131RootLayerData::DataPacket(DataPacketFramingLayer {
                source_name: source_name.to_string().into(),
                priority: E131_DEFAULT_PRIORITY,
                synchronization_address: 0,
                sequence_number,
                preview_data: false,
                stream_terminated: false,
                force_synchronization: false,
                universe,
                data: DataPacketDmpLayer {
                    property_values: property_values.to_vec().into(),
                },
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::build_data_packet;
    use sacn::packet::AcnRootLayerProtocol;
    use uuid::Uuid;

    #[test]
    fn build_data_packet_round_trips_with_property_values() {
        let packet = build_data_packet(
            Uuid::new_v4(),
            "Cohen Test",
            12,
            7,
            &[0, 11, 22, 33, 44, 55],
        );
        let packed = packet.pack_alloc().expect("packet should pack");
        let parsed = AcnRootLayerProtocol::parse(&packed).expect("packet should parse");

        assert_eq!(parsed, packet);
    }
}

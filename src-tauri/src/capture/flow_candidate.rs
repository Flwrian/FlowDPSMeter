//! Preserve pre-combat identities while proving a directional game connection.
use std::collections::VecDeque;
use super::{captured_payload::CapturedPayload, framing::{self,FrameKind},stream_processor::has_valid_party_roster};
use crate::pipeline::TcpOrder;

#[derive(Default)]
pub struct FlowCandidate {
    order:TcpOrder, buffer:Vec<u8>, signatures:VecDeque<i64>,
    packets:VecDeque<CapturedPayload>, bytes:usize,
    pub truncated:bool,
    pub last_seen:i64,
}
impl FlowCandidate {
    pub fn push(&mut self, packet:CapturedPayload)->bool {
        self.last_seen=packet.captured_at_ms;
        self.bytes+=packet.data.len();
        self.packets.push_back(packet.clone());
        // A quiet party may be formed long before the first attack. Keep its
        // identity packets independently of the three-second detection window.
        while self.bytes>1024*1024 || self.packets.len()>2048 ||
            self.packets.front().is_some_and(|p|self.last_seen-p.captured_at_ms>60_000) {
            if let Some(p)=self.packets.pop_front(){self.bytes-=p.data.len();self.truncated=true;}else{break;}
        }
        let old_losses=self.order.losses;
        let mut roster=false;
        for chunk in self.order.push(packet.tcp_seq,&packet.data,packet.captured_at_ms) {
            if self.order.losses>old_losses {self.buffer.clear();self.truncated=true;}
            if chunk.windows(3).any(|w|w==[0x0e,0,0x36]||w==[6,0,0x36]) {self.signatures.push_back(self.last_seen);}
            self.buffer.extend(chunk);
            let walk=framing::walk(&self.buffer);
            for frame in &walk.frames {roster|=inspect(frame.kind,frame.payload(&self.buffer),0);}
            self.buffer.drain(..walk.consumed);
            if self.buffer.len()>128*1024 {self.buffer.clear();self.truncated=true;}
        }
        while self.signatures.front().is_some_and(|t|self.last_seen-*t>3000){self.signatures.pop_front();}
        roster || self.signatures.len()>=12
    }
    pub fn take_packets(&mut self)->Vec<CapturedPayload>{self.bytes=0;self.packets.drain(..).collect()}
}
fn inspect(kind:FrameKind,data:&[u8],depth:usize)->bool {
    if kind==FrameKind::Packet {return has_valid_party_roster(data);}
    if depth>=framing::MAX_BUNDLE_DEPTH {return false;}
    let Some(inner)=framing::decompress_bundle(data) else{return false;};
    framing::walk_inner(&inner).frames.iter().any(|f|inspect(f.kind,f.payload(&inner),depth+1))
}

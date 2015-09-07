
state_packets!(
	handshake Handshaking {
		 serverbound Serverbound {
			// Handshake is the first packet sent in the protocol.
			// Its used for deciding if the request is a client
			// is requesting status information about the server
			// (MOTD, players etc) or trying to login to the server.
			//
			// The host and port fields are not used by the vanilla
			// server but are there for virtual server hosting to
			// be able to redirect a client to a target server with
			// a single address + port.
			//
			// Some modified servers/proxies use the handshake field
			// differently, packing information into the field other
			// than the hostname due to the protocol not providing
			// any system for custom information to be transfered
			// by the client to the server until after login.
			Handshake => 0 {
				// The protocol version of the connecting client
				protocol_version: VarInt,
				// The hostname the client connected to
				host: String,
				// The port the client connected to
				port: u16,
				// The next protocol state the client wants
				next: VarInt
			}
		}
		clientbound Clientbound {
		}
	}
	play Play {
		serverbound Serverbound {
		}
		clientbound Clientbound {
	   }
	}
	login Login {
		serverbound Serverbound {
		}
		clientbound Clientbound {
		}
	}
	status Status {
		serverbound Serverbound {
            // StatusRequest is sent by the client instantly after
            // switching to the Status protocol state and is used
            // to signal the server to send a StatusResponse to the
            // client
			StatusRequest => 0 {
				empty: Empty
			},
            // StatusPing is sent by the client after recieving a
            // StatusResponse. The client uses the time from sending
            // the ping until the time of recieving a pong to measure
            // the latency between the client and the server.
			StatusPing => 1 {
				ping: i64
			}
		}
		clientbound Clientbound {
			// StatusResponse is sent as a reply to a StatusRequest.
			// The Status should contain a json encoded structure with
			// version information, a player sample, a description/MOTD
			// and optionally a favicon.
			//
			// The structure is as follows
			//	 {
			//		 "version": {
			//			 "name": "1.8.3",
			//			 "protocol": 47,
			//		 },
			//		 "players": {
			//			 "max": 20,
			//			 "online": 1,
			//			 "sample": [
			//				 {"name": "Thinkofdeath", "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20"}
			//			 ]
			//		 },
			//		 "description": "Hello world",
			//		 "favicon": "data:image/png;base64,<data>"
			//	 }
            StatusResponse => 0 {
                status: String
            },
            // StatusPong is sent as a reply to a StatusPing.
            // The Time field should be exactly the same as the
            // one sent by the client.
            StatusPong => 1 {
                ping: i64
            }
	   }
	}
);

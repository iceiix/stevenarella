
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
			Handshake => 0x00 {
				// The protocol version of the connecting client
				protocol_version: VarInt =,
				// The hostname the client connected to
				host: String =,
				// The port the client connected to
				port: u16 =,
				// The next protocol state the client wants
				next: VarInt =,
			}
		}
		clientbound Clientbound {
		}
	}
	play Play {
		serverbound Serverbound {
			// TabComplete is sent by the client when the client presses tab in
			// the chat box.
			TabComplete => 0x00 {
				text: String =,
				has_target: bool =,
				target: Option<Position> = when(|p: &TabComplete| p.has_target == true),
			}
			// ChatMessage is sent by the client when it sends a chat message or
			// executes a command (prefixed by '/').
			ChatMessage => 0x01 {
				message: String =,
			}
			// ClientStatus is sent to update the client's status
			ClientStatus => 0x02 {
				action_id: VarInt =,
			}
			// ClientSettings is sent by the client to update its current settings.
			ClientSettings => 0x03 {
				locale: String =,
				view_distance: u8 =,
				chat_mode: u8 =,
				chat_colors: bool =,
				displayed_skin_parts: u8 =,
				main_hand: VarInt =,
			}
			// ConfirmTransactionServerbound is a reply to ConfirmTransaction.
			ConfirmTransactionServerbound => 0x04 {
				id: u8 =,
				action_number: i16 =,
				accepted: bool =,
			}
			// EnchantItem is sent when the client enchants an item.
			EnchantItem => 0x05 {
				id: u8 =,
				enchantment: u8 =,
			}
			// ClickWindow is sent when the client clicks in a window.
			ClickWindow => 0x06 {
				id: u8 =,
				slot: i16 =,
				button: u8 =,
				action_number: u16 =,
				mode: u8 =,
				clicked_item: ()=, // TODO
			}
		}
		clientbound Clientbound {
			ServerMessage => 15 {
				message: format::Component =,
				position: u8 =,
			}
	   }
	}
	login Login {
		serverbound Serverbound {
			// LoginStart is sent immeditately after switching into the login
			// state. The passed username is used by the server to authenticate
			// the player in online mode.
			LoginStart => 0 {
				username: String =,
			}
			// EncryptionResponse is sent as a reply to EncryptionRequest. All
			// packets following this one must be encrypted with AES/CFB8
			// encryption.
			EncryptionResponse => 1 {
				// The key for the AES/CFB8 cipher encrypted with the
				// public key
				shared_secret: LenPrefixed<VarInt, u8> =,
				// The verify token from the request encrypted with the
				// public key
				verify_token: LenPrefixed<VarInt, u8> =,
			}
		}
		clientbound Clientbound {
			// LoginDisconnect is sent by the server if there was any issues
			// authenticating the player during login or the general server
			// issues (e.g. too many players).
			LoginDisconnect => 0 {
				reason: format::Component =,
			}
			// EncryptionRequest is sent by the server if the server is in
			// online mode. If it is not sent then its assumed the server is
			// in offline mode.
			EncryptionRequest => 1 {
				// Generally empty, left in from legacy auth
				// but is still used by the client if provided
				server_id: String =,
				// A RSA Public key serialized in x.509 PRIX format
				public_key: LenPrefixed<VarInt, u8> =,
				// Token used by the server to verify encryption is working
				// correctly
				verify_token: LenPrefixed<VarInt, u8> =,
			}
			// LoginSuccess is sent by the server if the player successfully
			// authenicates with the session servers (online mode) or straight
			// after LoginStart (offline mode).
			LoginSuccess => 2 {
				// String encoding of a uuid (with hyphens)
				uuid: String =,
				username: String =,
			}
			// SetInitialCompression sets the compression threshold during the
			// login state.
			SetInitialCompression => 3 {
				// Threshold where a packet should be sent compressed
				threshold: VarInt =,
			}
		}
	}
	status Status {
		serverbound Serverbound {
            // StatusRequest is sent by the client instantly after
            // switching to the Status protocol state and is used
            // to signal the server to send a StatusResponse to the
            // client
			StatusRequest => 0 {
				empty: () =,
			}
            // StatusPing is sent by the client after recieving a
            // StatusResponse. The client uses the time from sending
            // the ping until the time of recieving a pong to measure
            // the latency between the client and the server.
			StatusPing => 1 {
				ping: i64 =,
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
                status: String =,
            }
            // StatusPong is sent as a reply to a StatusPing.
            // The Time field should be exactly the same as the
            // one sent by the client.
            StatusPong => 1 {
                ping: i64 =,
            }
	   }
	}
);

# SIF - Cardano Node Mempool Monitor and Global Pending Transaction Mempool Gateway 

This tool implements Pallas and the Ouroboros local tx monitoring mini-protocol. 
We are working on a global - pending - UTxOs / transaction monitoring solution to improve the transaction building process on Cardano. 

SIF runs next to a CardanoNode and publishs the UTxOs contained in the local mempool to a the SIF-Server system. 

Everbody is invited to help us devloping by running a local SIF node with their submit API relays.
The SIF-Server is still in development, however a first version is running already, but it needs further optimization to meet our expectations. 

SIF can also be used to just print your local mempool to the console. Furhter features like query the mempool will follow soon. 

The latest pre-release was tested by several SPOs, many thanks to Freeloaderz DAO.  

Settings

The only needed setting for SIF is the "CARDANO_NODE_SOCKET_PATH" as environment variable. 
You can set it by 'export CARDANO_NODE_SOCKET_PATH=<path/to/your/node.socket>'

The log output is controlled with the environment variable 'RUST_LOG', if you want to have a console output about your mempool status set: 
'export RUST_LOG=info'

The environment variable 'CARDANO_NETWORK' can be set to 'MAINNET', 'TESTNET' or a arbitrary network magic number as string to switch between networks.

The SIF-Server URL can be set by 'SIF_SERVER_URL', the default value points to our test system and there are no other systems available at the moment. 

If you want to query the Server you can do this with: 

http://3.133.230.181:32001/gutxomem 

To get UTxOs but we do not gurantee correctness at the moment as the system is still in development. 
Please do not use it for any dApps.  

Links: 

[ www.drasil.io ]
[ www.gomaestro.org ]
[ www.freeloaderz.io ]
[ https://github.com/txpipe/pallas ]



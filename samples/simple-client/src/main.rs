//! This is a sample OPC UA Client that connects to the specified server, fetches some
//! values before exiting.
extern crate opcua_types;
extern crate opcua_core;
extern crate opcua_client;

use opcua_client::prelude::*;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

// This simple client will fetch values and exit, or if --subscribe is passed on the command line
// it will subscribe to values and run indefinitely, printing out changes to the values.

fn main() {
    // Optional - enable OPC UA logging
    opcua_core::init_logging();

    // Test for --subscribe
    let args: HashSet<String> = env::args().collect();
    let subscribe_flag = args.contains("--subscribe");

    // Use the sample client config to set up a client. The sample config has a number of named
    // endpoints one of which is marked as the default.
    let mut client = Client::new(ClientConfig::load(&PathBuf::from("../client.conf")).unwrap());

    // Ask the server associated with the default endpoint for its list of endpoints
    let endpoints = client.get_server_endpoints();
    if endpoints.is_err() {
        println!("ERROR: Can't get endpoints for server, error - {:?}", endpoints.unwrap_err().description());
        return;
    }
    let endpoints = endpoints.unwrap();
    println!("Server has these endpoints:");
    for e in &endpoints {
        println!("  {} - {:?} / {:?}", e.endpoint_url, SecurityPolicy::from_str(e.security_policy_uri.as_ref()).unwrap(), e.security_mode);
    }

    // Create a session to the default endpoint. It has to match one received from the get_endpoints call
    if let Ok(session) = client.new_session(&endpoints) {

        // Important - the session you receive is reference counted because it is used by background
        // threads. You must only lock the session for as long as you need to use it and you should
        // release the lock (i.e. drop it) when your need to use it is over. The subscription thread
        // also needs access to the session to send and receive messages and won't
        // be able to if your lock is never released.

        {
            // Connect to the server
            let mut session = session.lock().unwrap();
            if let Err(result) = session.connect_and_activate_session() {
                println!("ERROR: Got an error while creating the default session - {:?}", result.description());
            }
        }

        // The --subscribe arg decides if code should subscribe to values, or just fetch those
        // values and exit
        let result = if subscribe_flag {
            subscribe(session)
        } else {
            read_values(session)
        };
        if let Err(result) = result {
            println!("ERROR: Got an error while performing action - {:?}", result.description());
        }
    } else {
        println!("ERROR: Sample client cannot create a session!");
    }
}

fn nodes_to_monitor() -> Vec<ReadValueId> {
    vec![
        ReadValueId::read_value(NodeId::new_string(2, "v1")),
        ReadValueId::read_value(NodeId::new_string(2, "v2")),
        ReadValueId::read_value(NodeId::new_string(2, "v3")),
        ReadValueId::read_value(NodeId::new_string(2, "v4")),
    ]
}

fn subscribe(session: Arc<Mutex<Session>>) -> Result<(), StatusCode> {
    // Create a subscription
    println!("Creating subscription");

    {
        let mut session = session.lock().unwrap();
        let subscription_id = session.create_subscription(1f64, 10, 30, 0, 0, true, |items| {
            println!("Got changes to items {:?}", items);
        })?;
        println!("Subscription id = {}", subscription_id);

        // Make requests for the items to create
        let read_nodes = nodes_to_monitor();
        let items_to_create: Vec<MonitoredItemCreateRequest> = read_nodes.into_iter().map(|read_node| {
            MonitoredItemCreateRequest::new(read_node, MonitoringMode::Reporting, MonitoringParameters {
                client_handle: 0,
                sampling_interval: 0f64,
                filter: ExtensionObject::null(),
                queue_size: 1,
                discard_oldest: true,
            })
        }).collect();

        println!("Creating monitored items");
        let response = session.create_monitored_items(subscription_id, items_to_create)?;
        println!("Creating monitored items {:?}", response);
    }

    // Loops for ever. The publish thread will call the callback with changes on the variables
    loop {
        {
            // Break the loop if connection goes down
            let session = session.lock().unwrap();
            if !session.is_connected() {
                println!("Connection to server broke, so terminating");
                break;
            }
        }

        // Main thread has nothing to do - just wait for publish events to roll in
        use std::thread;
        use std::time;
        thread::sleep(time::Duration::from_millis(1000));
    }

    Ok(())
}

fn read_values(session: Arc<Mutex<Session>>) -> Result<(), StatusCode> {
    // Fetch some values from the sample server
    let read_nodes = nodes_to_monitor();
    let data_values = {
        let mut session = session.lock().unwrap();
        session.read_nodes(&read_nodes)?.unwrap()
    };

    // Print the values out
    println!("Values from server:");
    for data_value in data_values.iter() {
        if let Some(ref value) = data_value.value {
            println!("Value = {:?}", value);
        } else {
            println!("Value not found, error: {}", data_value.status.as_ref().unwrap().description());
        }
    }

    Ok(())
}


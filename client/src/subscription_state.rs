use opcua_types::{Byte, Double, UInt32};
use std::collections::HashMap;
use subscription::*;

pub struct SubscriptionState {
    /// Subscriptions (key = subscription_id)
    subscriptions: HashMap<UInt32, Subscription>,
}

impl SubscriptionState {
    pub fn new() -> SubscriptionState {
        SubscriptionState {
            subscriptions: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool { self.subscriptions.is_empty() }

    pub fn subscription_exists(&self, subscription_id: UInt32) -> bool {
        self.subscriptions.contains_key(&subscription_id)
    }

    pub fn add_subscription(&mut self, subscription: Subscription) {
        self.subscriptions.insert(subscription.subscription_id(), subscription);
    }

    pub fn modify_subscription(&mut self, subscription_id: UInt32, publishing_interval: Double, lifetime_count: UInt32, max_keep_alive_count: UInt32, max_notifications_per_publish: UInt32, priority: Byte) {
        if let Some(ref mut subscription) = self.subscriptions.get_mut(&subscription_id) {
            subscription.set_publishing_interval(publishing_interval);
            subscription.set_lifetime_count(lifetime_count);
            subscription.set_max_keep_alive_count(max_keep_alive_count);
            subscription.set_max_notifications_per_publish(max_notifications_per_publish);
            subscription.set_priority(priority);
        }
    }

    pub fn delete_subscription(&mut self, subscription_id: UInt32) {
        self.subscriptions.remove(&subscription_id);
    }

    pub fn insert_monitored_items(&mut self, subscription_id: UInt32, items_to_create: Vec<CreateMonitoredItem>) {
        if let Some(ref mut subscription) = self.subscriptions.get_mut(&subscription_id) {
            subscription.insert_monitored_items(items_to_create);
        }
    }

    pub fn modify_monitored_items(&mut self, subscription_id: UInt32, items_to_modify: Vec<ModifyMonitoredItem>) {
        if let Some(ref mut subscription) = self.subscriptions.get_mut(&subscription_id) {
            subscription.modify_monitored_items(items_to_modify);
        }
    }

    pub fn delete_monitored_items(&mut self, subscription_id: UInt32, items_to_delete: Vec<UInt32>) {
        if let Some(ref mut subscription) = self.subscriptions.get_mut(&subscription_id) {
            subscription.delete_monitored_items(items_to_delete);
        }
    }
}

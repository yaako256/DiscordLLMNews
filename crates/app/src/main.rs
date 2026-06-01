fn main() {
  println!("Hello, world!");
  kernel::debug();
  shared::debug();
  config::debug();
  notifier::debug();
  infra::debug();
  logger::debug();
  news_fetch::debug();
  llm::debug();
}

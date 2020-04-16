/// Команды с клиента
use bytebuffer::ByteBuffer;
use traitcast::TraitcastFrom;

use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

pub mod create_game_object;
pub mod delete_game_object;
pub mod update_u64_counter;

/// Декодер входящей команды
pub trait C2SCommandDecoder {
	/// идентификатор команды
	const COMMAND_ID: u8;
	
	/// Декодирование команды
	/// bytes - массив байт, из которого будет прочитана информация
	/// если результат Option::None то указатель данных в bytes будет сброшен в начало
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>>;
}


/// Интерфейс команды с клиента
pub trait C2SCommandExecutor: TraitcastFrom {
	/// Выполнить команду
	fn execute(&self, client: &Client, room: &mut Room);
}

pub trait Context<'ctx> {
	type Fields: AsRef<[Field<'ctx>]>;

	fn to_fields(&'ctx self) -> Self::Fields;
}

pub struct Field<'ctx> {
	pub(crate) hash: u64,
	pub(crate) value: &'ctx str,
}

pub struct Monitor<'a> {
    pub temperatures: &'a mut [(&'static str, i16)],
}

impl<'a> Monitor<'a> {
    pub fn new(temperatures: &'a mut [(&'static str, i16)]) -> Self {
        Self { temperatures }
    }
}

impl<'a> sval::value::Value for Monitor<'a> {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(1))?;
        stream.map_key("temperatures")?;
        stream.map_begin(Some(self.temperatures.len()))?;
        for (name, value) in self.temperatures.iter() {
            stream.map_key(name)?;
            stream.map_value(value)?;
        }
        stream.map_end()
    }
}

impl<'a> core::fmt::Display for Monitor<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

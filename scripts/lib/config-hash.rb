# Helper class that enables easy use of default values with minimal fuss.  When a `ConfigHash`
# is created, it can be passed a Hash of default values to use when a requested value is not
# found.
class ConfigHash < Hash
  # @!attribute [r]
  #   Default values for this hash.
  #   @return [Hash]
  attr_reader :defaults

  # These methods have been made private since their semantics conflict with the purpose of
  # ConfigHash.
  private(:default, :default=)
  private(:default_proc=) if RUBY_VERSION >= '1.9.1'


  def clone()
    o = super()
    o.instance_variable_set(:@defaults, @defaults.clone)
    o
  end
  

  # Initialize a new `ConfigHash` instance.
  #
  # @param [ConfigHash,Hash,nil] defaults Hash containing default values to use when a given
  #     setting cannot be found, or `nil`.
  #
  # @param [Proc,#clone] default_default A proc to call or value to use as a
  #   default in the new ConfigHash when no default value is available in
  #   `defaults`.
  def initialize(_values = {}, _options = nil, &block)
    _defaults = {}

    default_proc = block if block_given?

    if _options.nil? and _values.has_key?(:defaults)
      _options = _values
      _values = {}
    end
    unless _options.nil?
      _defaults = _options[:defaults]
    end

    @defaults = ( _defaults.nil? ? {} : _defaults )
    @defaults.freeze
    update(_values)
  end

  def subhash_with_defaults(key, value = {})
    _defaults = @defaults[key]
    _klass = ConfigHash
    if _defaults.kind_of?(ConfigHash)
      # Allow for subclasses of ConfigHash in the defaults.
      _klass = _defaults.class
      _defaults = _defaults.defaults if _defaults.empty?
    end          

    _klass.new(value, defaults: _defaults)
  end

  # Setter method override.  This method allows setting values within a (sub) hash with
  # defaults, without losing access to the other default values.
  #
  # @param [Object] k Search key.
  # @param [Object] v Value.
  def []=(key, value)
    old_value = begin
                  fetch(key)
                rescue
                  nil
                end
    new_value =
      if value.kind_of?(Hash)
        if has_key?(key) and old_value.kind_of?(Hash)
          old_value.update(value)
        else
          store(key, subhash_with_defaults(key, value))
        end
      else
        store(key, value)
      end
    new_value
  end

  # Getter method override.  Attempts to use default values when a missing key is encountered,
  # and does some magic to enable assignment to sublevels of a (previously) entirely-default
  # ConfigHash tree.
  #
  # Assignment to sublevels is made possible by creation of an empty ConfigHash for each
  # non-existent key whose default value is a Hash instance.  (If instead the default value was
  # returned, any assignment would change the _default_ value!)
  #
  # @param [Object] s Search key.
  #
  # @return [Object] The value associated with `s`, or the default value if there is no such
  #    association in this hash.  If no default values have been set, returns `nil`.
  def [](key)
    if explicit_key?(key)
      super(key)
    else
      if (result = @defaults[key]).kind_of?(Hash)
        # We need to create the empty storage in case the client assigns an entry of the
        # returned hash -- otherwise, we'd get a modified _default_!
        store(key, subhash_with_defaults(key, result))
      else
        result
      end
    end
  end

  def fetch(key, *rest)
    if explicit_key?(key)
      super
    elsif @defaults.key?(key)
      @defaults[key]
    elsif ! rest.empty?
      rest.shift
    else
      raise KeyError.new('key not found: %s' % key.inspect)
    end
  end      
  # Recursively update from a source hash.  This allows e.g. missing values in the
  # `:directories' sub-hash of a Config.
  #
  # @param [Hash] from Source hash (to update with).
  #
  # @return [Hash] @c tgt
  def update(from)
    from.each_pair do |key, value|
      old_value = begin
                    fetch(key)
                  rescue
                    nil
                  end

      if value.kind_of?(Hash)
        if has_key?(key) and old_value.kind_of?(ConfigHash)
          old_value.update(value)
        else
          store(key, subhash_with_defaults(key, value))
        end
      else
        store(key, value)
      end
    end
    self
  end

  # # Proxy to allow dotted-object-style access to configuration values.
  def method_missing(sym)
    self[sym]
  end
  def respond_to_missing?(sym, *args)
    self.key?(sym) && args.empty?
  end

  alias_method(:explicit_key?, :key?)
  def key?(name)
    explicit_key?(name) or @defaults.key?(name)
  end
  

  def to_hash
    o = {}
    all_keys.each { |k|
      v = self[k]
      o[k] = v.kind_of?(Hash) ? v.to_hash : v
    }
    o
  end

  def ==(other)
    self === other or self.to_hash == other.to_hash
  end

  def ===(other)
    other.kind_of?(ConfigHash) and  @defaults === other.defaults
  end

  def inspect
    "#<#{self.class}:#{'%#x' % self.object_id.abs} #{super}>"
  end

  def all_keys
    (@defaults.respond_to?(:all_keys) ? @defaults.all_keys : @defaults.keys) | keys
  end
end

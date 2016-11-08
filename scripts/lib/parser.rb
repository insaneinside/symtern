module ReLemon
  # Error raised when problems are detected in the input grammar.
  class GrammarError < ::RuntimeError
    attr_reader :file, :line, :column
    def initialize(msg, file, line, column = nil)
      @file = file
      if column.nil?
        @line = line.line
        @column = line.column
      else
        @line = line
        @column = column
      end
      super('%s:%s:%s: %s' % [@file, @line, @column, msg])
    end
  end

  # Base parser class providing common functionality for both re2c and
  # Lemon parsers.
  module Parser
    # Convenient representation of a position in an input file.
    class FilePosition
      @line = nil
      @column = nil
      @offset = nil

      # Line number.
      #
      # @!attribute [r]
      #   @return [Integer]
      attr_reader :line

      # Column number.
      #
      # @!attribute [r]
      #   @return [Integer]
      attr_reader :column

      # Byte offset from the start of the file.
      # @!attribute [r]
      #   @return [Integer]
      attr_reader :offset

      # Initialize the file-position object, optionally specifying the initial
      # position within the file.
      #
      # @param [Integer] _line Initial line value.
      # @param [Integer] _column Initial column number.
      # @param [Integer] _offset Initial byte offset.
      def initialize(_line = nil, _column = nil, _offset = nil)
        @line = _line || 1
        @column = _column || 0
        @offset = _offset || 0
      end

      # Create an updated position object by examining a string of consumed
      # data and updating internal variables in a duplicated FilePosition
      # as necessary.
      #
      # @param [String] consumed_data Input data between the current position
      #     and the desired new position in the input.
      def update(consumed_data)
        self.dup.update!(consumed_data)
      end

      # Update the position in-place by examining a string of consumed data and
      # updating internal variables as necessary.
      #
      # @param [String] consumed_data Input data between the current position
      #     and the desired new position in the input.
      def update!(consumed_data)
        @offset += consumed_data.length

        @line += (nl = consumed_data.count("\n"))
        @column =
          if nl > 0
            /(?<=\n)?[^\n]*\z/.match(consumed_data)[0].length
          else
            @column + consumed_data.length
          end
        self
      end

      def inspect
        '#<%s:%#x %s>' % [self.class, self.object_id, self.to_s]
      end

      def to_s
        'l. %u/c. %u' % [@line, @column]
      end
    end


    # Token-instance data structure.
    TokenInstance = ::Struct.new(:type, :value, :string, :position)

    # Stores the information necessary to recognize, fetch, and create
    # a TokenInstance for a given token type in the input file.
    class TokenDefinition
      # Fetch procedure used if none is explicitly passed to {#initialize}.
      # This implementation simply uses the regular-expression passed to
      # {#initialize}.
      DEFAULT_FETCH_PROC = lambda { |input, m| input.slice!(m.regexp) }

      @type = nil
      @match_regexp = nil
      @fetch_proc = nil

      # The type of token that this object defines.
      #
      # @!attribute [r]
      #   @return [Symbol]
      attr_reader :type

      # The regular expression that, when it matches against the beginning of
      # an input string, will cause a parser to call this definition's
      # `fetch` method.
      #
      # @!attribute [r]
      #   @return [Regexp]
      attr_reader :match_regexp

      # Initialize a token definition, specifying the token type, initial
      # regular expression, and (optionally) a block or proc to be used for
      # fetching the data for this type of token.
      #
      # @param [Symbol] _type Token-type being defined.
      #
      # @param [Regexp] _regexp Regular expression that matches partial-initial
      #     or full instances of this token type.  If it matches only part of
      #     the full token string, a proc (`_fetch_proc`) or block must be
      #     passed to `#initialize`.
      #
      # @param [Proc,nil] _fetch_proc If non-`nil`, a procedure that accepts an
      #     input string and the MatchData from the definition's match regexp,
      #     and returns the initial part of the input string that represents
      #     this token.  It is expected that the proc will modify the input
      #     in-place, e.g. using {String#slice!}.
      def initialize(_type, _regexp, _fetch_proc = nil, &block)
        raise APIUsageError('TokenDefinition#initialize can only accept one block argument') if
          block_given? and not _fetch_proc.nil?
        if _fetch_proc.nil?
          _fetch_proc = block if block_given?
          _fetch_proc ||= DEFAULT_FETCH_PROC
        end

        @type = _type
        @match_regexp = _regexp
        raise APIUsageError.new('Invalid `fetch_proc` argument for %s#%s' %
                                [self.class.name, __method__]) unless
          _fetch_proc.kind_of?(Proc) or _fetch_proc.kind_of?(Regexp)

        @fetch_proc =
            case _fetch_proc
            when Proc
              _fetch_proc
            when Regexp
              proc { |input, m| input.slice!(_fetch_proc) }
            end
      end

      # Attempt to fetch an instance of this token type from the given input
      # string.  If the definition matches, removes the matching initial part
      # of the input, updates the given position, and returns a corresponding
      # TokenInstance; otherwise returns `nil`.
      #
      # @param [String] input Remaining input data.
      #
      # @param [FilePosition] _position Current position in the input file.
      #
      # @return [TokenInstance,nil] Instance of this token type found at the
      #     start of `input`, or `nil` if no such instance was found.
      def fetch(input, _position, _binding = self)
        if not (m = @match_regexp.match(input)).nil?
          tok_position = _position.dup

          pre_length = input.length
          value = _binding.instance_exec(input, m, &@fetch_proc)

          removed = value
          value, removed = value if value.kind_of?(Array)

          unless value.nil?
            # Modify the buffer if the fetch proc didn't.
            input.slice!(0, removed.length) if input.length == pre_length
            _position.update!(removed) # update file position
            return TokenInstance.new(@type, value, removed, tok_position)
          end
        else
          nil
        end
      end
    end

    # State data for a parser.
    class ParseState
      INPUT_TRIM_REGEXP = /\A[ \t\v\n]+/
      @input_string = nil
      @buffer = nil
      @filename = nil
      @position = nil
      @prev_token = nil


      # Grammar context into which parsed data is stored.
      #
      # @!attribute [r]
      #   @return [ReLemon::Context]
      attr_reader :context


      # Name of the file being parsed, if applicable.
      #
      # @!attribute [r]
      #   @return [String]
      attr_reader :filename

      # Portion of `input_string` that has yet to be tokenized or parsed.
      #
      # @!attribute [r]
      #   @return [String]
      attr_reader :buffer

      # The full, unmodified string on which a parse was requested.
      #
      # @!attribute [r]
      #   @return [String]
      attr_reader :input_string

      # Token returned by `next_token()` for the PREVIOUS run of
      # e.g. `handle_token()`.
      #
      # @!attribute [r]
      #   @return [TokenInstance]
      attr_reader :prev_token

      def remove_junk_from_top()
        unless (removed = @buffer.slice!(self.class.const_get(:INPUT_TRIM_REGEXP))).nil?
          @position.update!(removed)
        end
      end

      # Whether the state's unparsed-data buffer is empty.
      #
      # @attribute [r]
      #   @return [Boolean]
      def empty?
        @buffer.empty?
      end

      # Initialize a new parse-state object.
      #
      # @param [String] _input_string String to parse.
      #
      # @param [Hash] opts Options hash.
      # @option opts [ReLemon::Context] :context Context to store parsed data in.
      # @option opts [String] :filename File name to use when reporting errors
      # @option opts [Integer] :line Initial line number.
      # @option opts [Integer] :column Initial column number.
      # @option opts [Integer] :offset Initial byte offset.
      def initialize(_input_string, **opts)
        @input_options = opts
        @input_string = _input_string.clone.freeze
        @buffer = _input_string.clone
        @filename = opts[:filename]
        @position = opts[:position] || FilePosition.new(opts[:line], opts[:column], opts[:offset])

        @trim_input = opts.fetch(:trim_input, false)

        @prev_token = nil
        @token_set = nil

        @terminate_parse_requested = false
      end

      attr_accessor :token_set

      def terminate_parse
        @terminate_parse_requested = true
        [nil,'']
      end

      # Update the parser-state's `line` and `column` values given a string of
      # consumed data.
      def update_position(consumed_data)
        @position.update!(consumed_data)
      end

      # Get the line and column number corresponding to the first character in
      # the parse state's buffer.
      #
      # @return [Array<Integer>]
      def position()
        @position
      end

      def next_token()
        remove_junk_from_top() if @trim_input

        buf = @buffer
        return nil if buf.empty?

        o = nil
        @token_set.each { |defn| break unless (o = defn.fetch(buf, self.position, self)).nil? }
        if o.nil?
          lim = [buf.length, 32].min
          raise GrammarError.new('Parse error on "%s%s"' % [buf[0...lim], lim < buf.length ? '...' : ''],
                                 filename, self.position)
        else
          if @input_options[:dump_tokens]
            $stderr.puts(o.inspect)
            $stderr.flush()
          end
          return o
        end
      end



      # Run the parsing algorithm on the input string, and return the context.
      # @return [ReLemon::Context]
      def run()
        while not empty?
          last_buf = @buffer.clone
          tok = next_token()
          break if @terminate_parse_requested

          raise '>>>' + @buffer + '<<<' if @buffer.length == last_buf.length

          # It's possible the buffer was empty except for some whitespace,
          # which may have been discarded by `next_token`.
          handle_token(tok) unless tok.nil?
          @prev_token = tok
        end
        
        finish()
      end
    end

    # Define singleton methods on classes/modules that include Parser.
    def self.included(_module)
      _module.instance_exec do
        const_set(:FilePosition, ::ReLemon::Parser::FilePosition)
        const_set(:ParseState, Class.new(::ReLemon::Parser::ParseState) do |ps|
                    define_method(:initialize) do |*a, **opts|
                      super(*a, **opts)
                      @token_set = _module.const_get(:TOKENS)
                    end
                  end)

        # Parse a named file.
        def self.parse_file(file, **options)
          # begin
          #   oldwd = Dir.pwd
          #   Dir.chdir(File.dirname(file))

          self::ParseState.new(File.read(file),
                               options.merge(filename: file)).
            run()

          # ensure
          #   Dir.chdir(oldwd)
          # end
        end

        def self.parse_string(buf, **options)
          self::ParseState.new(buf, options.has_key?(:filename) ? options : options.merge(filename: '(buffer)')).
            run()
        end
      end
    end
    # class << self
    #   include IINet::Util::Memoizer
    #   def anchor_regexp(re)
    #     source = re.source
    #     if source =~ /\A\\A/ or source =~ /\A\^/
    #       re
    #     elsif full
    #       /\A#{source}\z/
    #     else
    #       /\A#{source}/
    #     end
    #   end
    #   memoize(:anchor)
    # end
  end
end

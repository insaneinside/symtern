require_relative('parser')
module Rust
  module R
    @group_name = '_g0'
    def self.gn()
      @group_name.succ!
    end
    class Escaped < String
    end

    # Escape appropriately for the input type.
    #   Escaped: return the input untouched
    #   String: return Escaped.new(Regexp.escape(input))
    #   Regexp (and everything else): return the input untouched.
    def self.escape(s)
      if s.kind_of?(Escaped)
        s
      elsif s.kind_of?(String)
        Escaped.new(Regexp.escape(s))
      else
        s
      end
    end
    
    IDENT = /[_a-zA-Z][_a-zA-Z0-9]*/

    def self.list(sep, pat)
      sep = escape(sep)
      
      pat1, pat2 = pat.respond_to?(:call) \
        ? [pat.call(), pat.call()] \
        : [pat, pat]
      
      /#{pat1}(?:\s*#{sep}\s*#{pat2})*/m
    end

    def self.balanced(s, name)
      a, b = [Regexp.escape(s[0]), Regexp.escape(s[1])]
      /(?<#{name}>#{a}(?:[^#{a}#{b}]|\g<#{name}>)*#{b})/m
    end

    def self.generics(name = 'generics')
      self.balanced('<>', name)
    end
    def self.block(name = nil)
      name = 'block' if name.nil?
      self.balanced('{}', name)
    end
    def self.path()
      /(?:[!?]\s*)?(?:::)?#{list('::', lambda { || /#{IDENT}|#{generics(gn())}|#{IDENT}\s*#{generics(gn())}/ })}/
    end
    def self.ident_or_path()
      /'#{IDENT}|#{path()}/
    end
    def self.bounds_list()
      /#{list('+', lambda { || ident_or_path() })}/m
    end
    def self.bounded(thing = nil)
      thing = escape(thing)
      thing ||= ident_or_path()
      /#{thing}\s*:\s*#{self.bounds_list()}/
    end

    def self.where_clause(); /where\s+#{list(',', lambda { || bounded()})}/m; end

    def self.item_with_generics(keyword, name = IDENT)
      kw = escape(keyword)
      name = escape(name)
      /(?:pub\s+)?#{kw}\s+(?<#{keyword}_name>#{name})(?:\s*#{generics(gn())})?/
    end
      
    def self.fn(name = IDENT)
      /#{item_with_generics('fn', name)}\s*#{balanced('()', 'arguments')}(?:\s*->\s*#{path()})?(?:\s*#{where_clause()})?\s*#{block()}/m
    end

    def self.trait(name = IDENT)
      name = escape(name)
      /#{item_with_generics('trait', name)}(?:\s*:\s*#{bounds_list()})?(?:\s*#{where_clause()})?\s*#{block()}/m
    end
    TRAIT = trait()

    BLOCK = block()

    IMPL = /impl#{generics(gn())}?\s+#{path()}(?:\s+for\s+#{path()})?(?:\s+#{where_clause})?\s*#{BLOCK}/m

    EXTERN_CRATE = /extern\s+crate\s+(?<crate>#{IDENT})(?:\s+as\s+(?<alias>#{IDENT}))?\s*;/m
    MOD = /(?:pub\s+)?mod\s+(?<mod>#{IDENT})\s*(?:#{BLOCK}|;)/

    IDENT_AS_IDENT = /#{IDENT}(?:\s+as\s+#{IDENT})?/
    USE_DECL = /(?:pub\s+)?use\s+#{path()}(?:\s*::\s*(?:\*|#{IDENT_AS_IDENT}|\{\s*#{list(/\s*,\s*/, IDENT_AS_IDENT)}\s*\}))?\s*;/m


    FN = self.fn()
    ATTRIBUTE = /\#!?#{balanced('[]', gn())}/m
    STRUCT_OR_ENUM = /(?:pub\s+)?(?:struct|enum)\s+#{IDENT}(?:\s*#{generics(gn())})?(?:\s*#{where_clause})?\s*#{BLOCK}/
    NEWTYPE = /(?:pub\s+)?struct\s+#{IDENT}(?:\s*#{generics(gn())})?\s*#{balanced('()', gn())}(?:\s*#{where_clause})?\s*;/
    TYPE_DEFN = /#{STRUCT_OR_ENUM}|#{NEWTYPE}/
    TYPE_ALIAS = /(?:pub\s+)?(?:type)\s+#{IDENT}(?:\s*#{generics})?\s*=\s*#{path()}\s*;/m

    BLOCK_COMMENT_CONTENTS = %r{[^*/]|\*(?!/)|(?<!\*)/}
    LINE_COMMENT_CONTENTS = /[^\n]/

    def self.expr(bn = nil)
      /(?:#{block(bn)}|[^\{\};])*/m
    end
    EXPR = expr()
    LET_DECL = /let(?:\s+mut)?\s+#{IDENT}(?:\s*:\s*#{path()})?\s*=\s*#{expr(gn())}/m
    MEMBER_ACCESS = /#{IDENT}\s*\.\s*#{IDENT}/m
    FN_CALL = /(?:#{path()}|#{MEMBER_ACCESS})\s*#{balanced('()', gn())}/m
    MACRO_CALL = /#{IDENT}\s*!\s*#{Regexp.union(balanced('()', gn()), balanced('[]', gn()), balanced('{}', gn()))}/m

    LOOP = /(?:'#{IDENT}\s*:\s*)?loop\s*#{block('loop_block')}|while\s*#{expr(gn())}\s*#{block('while_block')}/m

    IFCHAIN = /(?<ifchain>if\s+#{expr(gn())}\s*#{block(gn())}(?:\s*else(?:\s+\g<ifchain>|\s*#{block(gn())}))?)/m
    MATCH = /match\s*#{expr(gn())}\s*#{block(gn())}/
    
    STMT = /#{LET_DECL}|#{EXPR}|#{LOOP}|#{IFCHAIN}|#{MATCH}/

    def self.block_comment(contents = nil)
      if contents.nil?
        name = gn()
        %r{(?<#{name}>/\*(?:#{BLOCK_COMMENT_CONTENTS}|\g<#{name}>)*\*/)}m
      else
        contents = escape(contents)
        %r{/\*#{contents}\*/}
      end
    end
    def self.line_comment(contents = nil)
      if contents.nil?
        contents = /#{LINE_COMMENT_CONTENTS}*/
      else
        contents = escape(contents)
      end
      %r{//#{contents}\n}
    end

    def self.comment(contents = nil)
      /#{block_comment(contents)}|#{line_comment(contents)}/
    end
    
    COMMENT = /#{block_comment()}|#{line_comment()}/
    INNER_LINE_COMMENT = line_comment(/!#{LINE_COMMENT_CONTENTS}*/)
    INNER_BLOCK_COMMENT = block_comment(%r{!#{BLOCK_COMMENT_CONTENTS}*})
    INNER_COMMENT = /#{INNER_BLOCK_COMMENT}|#{INNER_LINE_COMMENT}/
    ELLIPSIS_COMMENT = comment(/\s*...\s*/)


    TAG_ID = /[-a-zA-Z][-.a-zA-Z0-9]*/
    def self.id_tag(id = TAG_ID)
      /id\s*=\s*#{id}/
    end

    def self.tag_comment(tag)
      tag = escape(tag)
      self.comment(/`\s*#{tag}\s*/)
    end

    def self.tagged_code_block(tag)
      /#{tag_comment(tag)}\s*#{BLOCK}/m
    end

    def self.tagged_comment_block(tag)
      tag = escape(tag)
      start = tag_comment(/#{tag}\s*\{/)
      _end = tag_comment('}')
      %r{#{start}(?<tcb_contents>(?:[^\{\}]|#{block('tcb')})*)#{_end}}m
    end

    def self.tagged_block(tag)
      Regexp.union(tagged_code_block(tag), tagged_comment_block(tag))
    end

    def self.anchored(re); /\A#{re}/; end

    def self.all_matches(re, s, ofs = 0)
      o = []
      while ofs < s.length && ! (m = re.match(s, ofs)).nil?
        o << m
        ofs = m.end(0)
      end
      o
    end
  end

  ExternCrateBase = ::Struct.new(:crate, :alias)
  class ExternCrate < ExternCrateBase
    def effective_name
      if self.alias.nil?
        self.crate
      else
        self.alias
      end
    end
  end
  class Parser
    include ReLemon::Parser
    TOKENS =
      [
       [:use, R.anchored(R::USE_DECL)],
       [:comment, R.anchored(R::COMMENT)],
       [:extern_crate, R.anchored(R::EXTERN_CRATE), proc do |_, m|
          [ExternCrate.new(m[:crate], m[:alias]), m[0]]
        end],
       [:mod, R.anchored(R::MOD)],
       [:attribute, R.anchored(R::ATTRIBUTE)],
       [:trait, R.anchored(R::TRAIT)],
       [:fn, R.anchored(R::FN)],
       [:fn_call, R.anchored(R::FN_CALL)],
       [:macro_stmt, R.anchored(/#{R::MACRO_CALL}\s*;/)],
       [:macro_invocation, R.anchored(R::MACRO_CALL)],
       [:type, R.anchored(R::TYPE_DEFN)],
       [:type_alias, R.anchored(R::TYPE_ALIAS)],
       [:impl, R.anchored(R::IMPL)],
       [:let, R.anchored(/#{R::LET_DECL}\s*;/)],
       [:space, /\A\s+/m]
      ].collect { |ary| TokenDefinition.new(*ary) }

    class ParseState < ::ReLemon::Parser::ParseState
      def initialize(*args)
        super
        @token_set = TOKENS
        @items = []
      end
      def next_token(*args)
        super(*args)
      end
      def handle_token(tok)
        @items << tok
      end
      def finish()
        @items
      end
    end
  end
end

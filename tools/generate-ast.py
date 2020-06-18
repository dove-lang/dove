class AstGenerator:
    def __init__(self, output_dir):
        self.output_dir = output_dir

    def run(self):
        self.define_ast("expr", [
            "Assign   : Token name, Box<dyn_Expr> value",
            "Binary   : Box<dyn_Expr> left, Token operator, Box<dyn_Expr> right",
            "Grouping : Box<dyn_Expr> expression",
            "Literal  : Literals value",
            "Unary    : Token operator, Box<dyn_Expr> right",
            "Variable : Token name"
        ])

    def define_ast(self, base_name, types):
        path = self.output_dir + base_name + '.rs'
        f = open(path, 'w+')

        f.write('use crate::token::*;\n\n')

        f.write('trait Expr {\n')
        f.write('    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result;\n')
        f.write('}\n\n')

        f.write('trait Visitor {\n')
        f.write('    type Result;\n\n')
        for type_ in types:
            type_name = type_.split(':')[0].strip()
            f.write('    fn visit_' + type_name.lower() + '_expr(&mut self, ' + type_name.lower() +
                    ': &' + type_name + ') -> Self::Result;\n')
        f.write('}\n\n')

        for type_ in types:
            type_split = type_.split(':')
            type_name, fields = type_split[0].strip(), type_split[1].strip().split(',')

            # Write basic struct and its fields.
            f.write('pub struct ' + type_name + ' {\n')
            for field in fields:
                field_type, field_name = field.strip().split(' ')
                if field_type == 'Box<dyn_Expr>':
                    field_type = 'Box<dyn Expr>'
                f.write('    pub ' + field_name + ': ' + field_type + ',\n')
            f.write('}\n\n')

            # Write struct implementation of the Expr trait.
            f.write('impl Expr for ' + type_name + ' {\n')
            f.write('    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {\n')
            f.write('        visitor.visit_' + type_name.lower() + '_expr(self)\n')
            f.write('    }\n')
            f.write('}\n\n')


if __name__ == '__main__':
    ast_generator = AstGenerator('../src/')
    ast_generator.run()

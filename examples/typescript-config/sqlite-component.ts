import * as wick from '../../crates/wick/wick-config/typescript/v1';

const db_addr = new wick.ResourceBinding(
  'DBADDR',
  new wick.Url('file://{{ ctx.root_config.db_file }}')
);

const get_user = new wick.SqlQueryOperationDefinition(
  'get_user',
  'SELECT * FROM users WHERE id = ${id}'
).inputs([new wick.Field('id', 'number')]);

const set_user = new wick.SqlQueryOperationDefinition(
  '_user',
  'INSERT INTO users(name, email) VALUES (${name}, ${email}) RETURNING *'
).inputs([new wick.Field('name', 'string'), new wick.Field('email', 'string')]);

const component = new wick.SqlComponent(db_addr.getName())
  .with([new wick.Field('db_file', 'string')])
  .operations([get_user, set_user]);

let test_group = new wick.TestConfiguration()
  .with({
    db_file: '{{ctx.env.SQLITE_DB}}',
  })
  .cases([
    new wick.TestDefinition('_user')
      .inputs([
        new wick.SuccessPacket('name', 'TEST_NAME'),
        new wick.SuccessPacket('email', 'TEST_EMAIL@example.com'),
      ])
      .outputs([
        new wick.PacketAssertionDef('output').assertions([
          new wick.PacketAssertion(wick.AssertionOperator.Contains, {
            email: 'TEST_EMAIL@example.com',
            name: 'TEST_NAME',
          }),
        ]),
      ]),
  ]);

export const config = new wick.ComponentConfiguration(component)
  .name('my_component')
  .resources([db_addr])
  .tests([test_group]);

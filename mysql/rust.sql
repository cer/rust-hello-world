use eventuate;


create table customers(
  id bigint not null auto_increment,
  name varchar(255) not null,
  credit_limit bigint not null,
  primary key(id)
  ) engine = InnoDB;


create table foo(id int not null);

insert into foo values(1);
insert into foo values(3);
insert into foo values(9);


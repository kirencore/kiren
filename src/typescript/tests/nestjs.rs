use crate::typescript::*;

#[test]
fn test_nestjs_controller() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
@Controller('users')
export class UserController {
    @Get()
    getUsers(): User[] {
        return [];
    }
    
    @Post()
    createUser(@Body() userData: CreateUserDto): User {
        return userData;
    }
}
"#;

    let result = transpiler.transpile_nestjs(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_nestjs_service() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
@Injectable()
export class UserService {
    @Inject('USER_REPOSITORY')
    private userRepository: Repository<User>;
    
    async findAll(): Promise<User[]> {
        return this.userRepository.find();
    }
}
"#;

    let result = transpiler.transpile_nestjs(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_nestjs_module() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
@Module({
    imports: [TypeOrmModule.forFeature([User])],
    controllers: [UserController],
    providers: [UserService],
    exports: [UserService],
})
export class UserModule {}
"#;

    let result = transpiler.transpile_nestjs(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_nestjs_dto() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
export class CreateUserDto {
    @IsString()
    @IsNotEmpty()
    name: string;
    
    @IsEmail()
    email: string;
    
    @IsOptional()
    @IsNumber()
    age?: number;
}
"#;

    let result = transpiler.transpile_nestjs(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_nestjs_guard() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
@Injectable()
export class AuthGuard implements CanActivate {
    canActivate(
        context: ExecutionContext,
    ): boolean | Promise<boolean> | Observable<boolean> {
        const request = context.switchToHttp().getRequest();
        return this.validateRequest(request);
    }
    
    private validateRequest(request: any): boolean {
        return true;
    }
}
"#;

    let result = transpiler.transpile_nestjs(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_nestjs_interceptor() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
@Injectable()
export class LoggingInterceptor implements NestInterceptor {
    intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
        const now = Date.now();
        return next.handle().pipe(
            tap(() => console.log(`After... ${Date.now() - now}ms`))
        );
    }
}
"#;

    let result = transpiler.transpile_nestjs(typescript);
    assert!(result.is_ok());
}
